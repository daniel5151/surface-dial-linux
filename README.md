# surface-dial-linux

A Linux userspace controller for the [Microsoft Surface Dial](https://www.microsoft.com/en-us/p/surface-dial/925r551sktgn). Requires Linux Kernel 4.19 or higher.

**DISCLAIMER: This software is still under active development!**

Things will change.
Things will break.
Things are probably buggy.

Bug reports are appreciated!

## Overview

`surface-dial-daemon` is a background daemon which receives raw events from the surface dial, and translates them to various actions.

The daemon uses FreeDesktop notifications to provide visual feedback when switching between actions.

![](notif-demo.gif)

It would be cool to create some sort of GUI overlay (similar to the Windows one), though that's out of scope at the moment.

## Implementation

Core functionality is provided by the following libraries.

- `libudev` to monitor when the dial connects/disconnects.
- `libevdev` to read events from the surface dial through `/dev/input/eventXX`, and to fake input through `/dev/uinput`.
- `hidapi` to configure dial sensitivity + haptics.
- `notify-rust` to send desktop notifications over D-Bus.

While the device-handling code itself is somewhat messy at the moment, it should be really easy to add new operating modes. Just add a new mode implementation under `src/controller/controls` (making sure to update `src/controller/controls/mod.rs`), and add it to the list of available modes in `main.rs`!

## Functionality

- [x] Interpret raw Surface Dial event
- Operating Modes
    - [x] Volume Controls
    - [x] Media Controls
    - [x] Scrolling - using a virtual mouse-wheel
    - [ ] Scrolling - using a virtual touchpad (for [smoother scrolling](https://who-t.blogspot.com/2020/04/high-resolution-wheel-scrolling-in.html)) - **WIP**
    - [x] Zooming
    - [x] [Paddle](https://www.google.com/search?q=arkanoid+paddle) (emulated left, right, and space key)
    - [ ] \(meta\) custom modes specified via config file(s)
- [x] Dynamically switch between operating modes
    - [x] Using a long-press activated "meta" mode
    - [ ] Context-sensitive (based on currently open application)
- [x] Mode Persistence (keep mode when dial disconnects)
- [x] Haptic Feedback
    - https://docs.microsoft.com/en-us/windows-hardware/design/component-guidelines/radial-controller-protocol-implementation
    - https://www.usb.org/sites/default/files/hutrr63b_-_haptics_page_redline_0.pdf
    - https://www.usb.org/sites/default/files/hut1_21.pdf
    - _This was tricky to figure out, but in the end, it was surprisingly straightforward! Big thanks to [Geo](https://www.linkedin.com/in/geo-palakunnel-57718245/) for pointing me in the right direction!_
- [x] Visual Feedback
    - [x] FreeDesktop Notifications

Feel free to contribute new features!

## Dependencies

Building `surface-dial-daemon` requires the following:

- Linux Kernel 4.19 or higher
- A fairly recent version of the Rust compiler
- `libudev`
- `libevdev`
- `hidapi`

You can install Rust through [`rustup`](https://rustup.rs/).

Unless you're a cool hackerman, the easiest way to get `libudev`, `libevdev`, and `hidapi` is via your distro's package manager.

```bash
# e.g: on ubuntu
sudo apt install libevdev-dev libhidapi-dev libudev-dev
```

## Building

`surface-dial-daemon` uses the standard `cargo build` flow.

```bash
cargo build -p surface-dial-daemon --release
```

The resulting binary is output to `target/release/surface-dial-daemon`

## Running

The daemon is able to handle the dial disconnecting/reconnecting, so as long as it's running in the background, things should Just Work:tm:.

Note that the daemon must run as a _user process_ (**not** as root), as it needs access to the user's D-Bus to send notifications.

Having to run as a user process complicates things a bit, as the daemon must be able to access several restricted-by-default devices under `/dev/`. Notably, the `/dev/uinput` device will need to have it's permissions changed for things to work correctly. The proper way to do this is using the included [udev rule](https://wiki.debian.org/udev), though if you just want to get something up and running, `sudo chmod 666 /dev/uinput` should work fine (though it will revert back once you reboot!).

See the Installation instructions below for how to set up the permissions / udev rules.

During development, the easiest way to run `surface-dial-linux` is using `cargo`:

```bash
cargo run -p surface-dial-daemon
```

Alternatively, you can run the daemon directly using the executable at `target/release/surface-dial-daemon`.

## Installation

I encourage you to tweak the following setup procedure for your particular linux configuration.

The following steps have been tested working on Ubuntu 20.04/20.10.

```bash
# Install the `surface-dial-daemon` (i.e: build it, and place it under ~/.cargo/bin/surface-dial-daemon)
# You could also just copy the executable from /target/release/surface-dial-daemon to wherever you like.
cargo install --path .

# IMPORTANT: modify the .service file to reflect where you placed the `service-dial-daemon` executable.
# if you used `cargo install`, this should be as simple as replacing `danielprilik` with your own user id
vi ./install/surface-dial.service

# create new group for uinput
# (the 99-uinput.rules file changes the group of /dev/uinput to this new group)
sudo groupdadd -f uinput

# add self to the new uinput group and the existing /dev/input group
sudo gpasswd -a $(whoami) uinput
sudo gpasswd -a $(whoami) $(stat -c "%G" /dev/input/event0)

# install the systemd user service
mkdir -p ~/.config/systemd/user/
cp ./install/surface-dial.service ~/.config/systemd/user/surface-dial.service

# install the udev rule
sudo cp ./install/99-uinput.rules /etc/udev/rules.d/99-uinput.rules

# reload systemd + udev
systemctl --user daemon-reload
sudo udevadm control --reload

# enable and start the user service
systemctl --user enable surface-dial.service
systemctl --user start surface-dial.service
```

To see if the service is running correctly, run `systemctl --user status surface-dial.service`.

You may need to reboot to have the various groups / udev rules propagate.

If things aren't working, feel free to file a bug report!

_Call for Contributors:_ It would be awesome to have a proper rpm/deb package as well.
