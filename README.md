# surface-dial-linux

A Linux userspace controller for the [Microsoft Surface Dial](https://www.microsoft.com/en-us/p/surface-dial/925r551sktgn). Requires Linux Kernel 4.19 or higher.

Bug reports and PRs are appreciated!

## Overview

`surface-dial-daemon` receives raw events from the surface dial and translates them to more conventional input events.

The daemon uses FreeDesktop notifications to provide visual feedback when switching between actions.

![](notif-demo.gif)

It would be cool to create some sort of GUI overlay (similar to the Windows one), though that's out of scope at the moment.

### Operating Modes

Holding down the button for \~750ms will open up a meta-menu (using FreeDesktop notifications) which allows you to switch between operating modes on-the-fly. The daemon persists the last selected mode on-disk, so if you only care about one mode (e.g: scrolling), you can just set it and forget it.

Modes in **bold** should be considered **EXPERIMENTAL**. They seem to work alright (some of the time), but they would really benefit from some more polish / bug fixes.

| Mode                         | Click             | Rotate               | Notes                                                                                  |
| ---------------------------- | ----------------- | -------------------- | -------------------------------------------------------------------------------------- |
| Scroll                       | -                 | Scroll               | Fakes chunky mouse-wheel scrolling <sup>1</sup>                                        |
| **Scroll (Fake Multitouch)** | Reset Touch Event | Scroll               | Fakes smooth two-finger scrolling                                                      |
| Zoom                         | -                 | Zoom                 |                                                                                        |
| Volume                       | Mute              | Volume               |                                                                                        |
| Media                        | Play/Pause        | Next/Prev Track      |                                                                                        |
| Media + Volume               | Play/Pause        | Volume               | Double-click = Next Track                                                              |
| **Paddle Controller**        | Space             | Left/Right Arrow Key | Play [arkanoid](https://www.google.com/search?q=arkanoid+paddle) as the devs intended! |

<sup>1</sup> At the time of writing, almost all Linux userspace programs don't take advantage of the newer high-resolution scroll wheel events, and only support the older, chunkier scroll wheel events. Check out [this blog post](https://who-t.blogspot.com/2020/04/high-resolution-wheel-scrolling-in.html) for more details.

### Custom Modes

At the moment, all modes (along with their meta-menu ordering) are hard-coded into the daemon itself.

If you don't mind hacking together a bit of [very simple] Rust code, adding new modes should be fairly straightforward - just add a new `ControlMode` implementation under `src/controller/controls` and instantiate it in `main.rs`.

If you ended up implementing new mode you think others would find useful, please consider upstreaming it!

## Building

Building `surface-dial-daemon` requires the following:

-   Linux Kernel 4.19 or higher
-   A fairly recent version of the Rust compiler
-   `libudev`
-   `libevdev`
-   `hidapi`

You can install Rust through [`rustup`](https://rustup.rs/).

Unless you're a cool hackerman, the easiest way to get `libudev`, `libevdev`, and `hidapi` is via your distro's package manager.

```bash
# e.g: on ubuntu
sudo apt install libevdev-dev libhidapi-dev libudev-dev
```

On certain Ubuntu distros, you may also need to install the `librust-libdbus-sys-dev` package:

```bash
sudo apt install librust-libdbus-sys-dev
```

Once those are installed, `surface-dial-daemon` can be built using the standard `cargo build` flow.

```bash
cargo build -p surface-dial-daemon --release
```

The resulting binary is output to `target/release/surface-dial-daemon`

## Running

The daemon is able to handle the dial disconnecting/reconnecting, so as long as it's running in the background, things should Just Work:tm:.

Note that the daemon must run as a _user process_ (**not** as root), as it needs access to the user's D-Bus to send notifications.

Having to run as a user process complicates things a bit, as the daemon must be able to access several restricted-by-default devices under `/dev/`. Notably, the `/dev/uinput` device and the Surface Dial's `/dev/hidrawX` device will need to have their permissions changed for things to work correctly. The proper way to do this is using the included [udev rules](https://wiki.debian.org/udev), though if you just want to get something up and running, `sudo chmod 666 <device>` should work fine (though it will revert back once you reboot!).

See the Installation section below for how to set up the permissions / udev rules.

During development, the easiest way to run `surface-dial-linux` is using `cargo`:

```bash
cargo run -p surface-dial-daemon
```

Alternatively, you can run the daemon directly using the executable at `target/release/surface-dial-daemon`.

## Installation

I encourage you to tweak the following setup procedure for your particular Linux configuration.

The following steps have been tested working on Ubuntu 20.04/20.10.

```bash
# Install the `surface-dial-daemon` (i.e: build it, and place it under ~/.cargo/bin/surface-dial-daemon)
# You could also just copy the executable from /target/release/surface-dial-daemon to wherever you like.
cargo install --path .

# add self to the existing /dev/input group (either `input` or `plugdev`, depending on your distro)
sudo gpasswd -a $(whoami) $(stat -c "%G" /dev/input/event0)

# install the systemd user service
mkdir -p ~/.config/systemd/user/
cp ./install/surface-dial.service ~/.config/systemd/user/surface-dial.service

# install the udev rules
sudo cp ./install/10-uinput.rules /etc/udev/rules.d/10-uinput.rules
sudo cp ./install/10-surface-dial.rules /etc/udev/rules.d/10-surface-dial.rules

# reload systemd + udev
systemctl --user daemon-reload
sudo udevadm control --reload

# enable and start the user service
systemctl --user enable surface-dial.service
systemctl --user start surface-dial.service
```

To see if the service is running correctly, run `systemctl --user status surface-dial.service`.

You may need to reboot to have the various groups / udev rules propagate.  You may also need to change DisableSecurity to DisableSecurity=true in /etc/bluetooth/network.conf to successfully pair the Surface Dial.

If things aren't working, feel free to file a bug report!

_Call for Contributors:_ It would be awesome to have a proper packaging pipeline set up as well (e.g for deb/rpm).

## Implementation Notes

Core functionality is provided by the following libraries.

-   `libudev` to monitor when the dial connects/disconnects.
-   `libevdev` to read events from the surface dial through `/dev/input/eventXX`, and to fake input through `/dev/uinput`.
-   `hidapi` to configure dial sensitivity + haptics.
-   `notify-rust` to send desktop notifications over D-Bus.

The code makes heavy use of threads + channels to do non-blocking event handling. While async/await would have been a great fit for an application like this, I optimized for getting something up-and-running rather than maximum performance. Plus, the Rust wrappers around `libudev`, `libevdev`, and `hidapi` don't have native async/await support, so it would have been a lot more work for not too much gain.

The codebase is reasonably well organized, aside from the `dial_device` implementation, which is admittedly a bit gnarly. There's a bit of of thread/channel spaghetti going on to ensure that the lifetime of the haptics object lines up with the lifetime of the `libevdev` objects (as reported by `libudev`). All things considered, it's not _too_ messy, but it could certainly use some cleanup. Fortunately, if you're only interested in implementing new operating modes, you won't have to worry about any of that, as all the nitty-gritty device interaction is neatly encapsulated behind the `ControlMode` trait.

## Feature Roadmap

This is a rough outline of features I'd like to see implemented in this daemon. There's a non-zero chance that at some point the daemon will be "good enough" for me, and some features will be left unimplemented.

Contributions are more than welcome!

-   [x] Interpreting raw Surface Dial events
-   [x] Haptic Feedback
    -   https://docs.microsoft.com/en-us/windows-hardware/design/component-guidelines/radial-controller-protocol-implementation
    -   https://www.usb.org/sites/default/files/hutrr63b_-_haptics_page_redline_0.pdf
    -   https://www.usb.org/sites/default/files/hut1_21.pdf
    -   _This was tricky to figure out, but in the end, it was surprisingly straightforward! Big thanks to [Geo](https://www.linkedin.com/in/geo-palakunnel-57718245/) for pointing me in the right direction!_
-   [x] Set up a framework to easily implement various operating modes
    -   [x] In-code abstraction over Surface Dial Events / Haptics API
    -   [ ] Config file(s) to create simple custom modes
-   [x] Dynamically switching between operating modes
    -   [x] Using a long-press activated "meta-mode"
    -   [ ] Context-sensitive (based on the currently open application)
-   [ ] Config-file support
    -   [ ] Adjusting timings (e.g: long press timeout, double-click window, etc...)
    -   [ ] Custom operating mode ordering in the meta-menu
-   [x] Visual Feedback
    -   [x] FreeDesktop Notifications
    -   [ ] \(longshot\) Windows-like Wheel menu
-   [x] Last selected mode persistence (between daemon invocations)
-   [x] Gracefully handling disconnect/reconnect
-   [ ] Packaging pipeline (e.g: deb/rpm)
