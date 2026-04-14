# Launchpad

![Launchpad Cover](./docs/cover.png)

I use Arch, btw. Since my system is as minimalist as possible, I had to ditch some standard UI tools for practical reasons. I don't run waybar (or any equivalent), which frees up a ton of screen real estate for my apps, but it deprives me of a convenient way to check the date, time, battery percentage, or manage notifications. And constantly typing `date`, `cal -m`, `btop`, etc., in the terminal isn't always the best UX.

So, I decided to kill several birds with one stone. I didn't spend much time thinking of a name, but I made up for it with its functionality and aesthetics. It’s a launcher, similar to wofi/rofi, but written in Rust and much better (at least for my specific workflow). Under the hood, it uses `relm4` (GTK4) and `gtk4-layer-shell`.

Besides looking great, I added the ability to manage notifications, check the date, time, and calendar, and get detailed hardware info about the battery state. As a bonus, it natively integrates with `libqalculate`, allowing you to quickly calculate, convert, and manipulate various units right from the search bar.

## Features

* **Smart Launcher:** Fast search through `.desktop` files. Applications are sorted by launch frequency first, and alphabetically second.
* **Seamless Calculator:** The moment you type a query that matches no apps, the interface automatically falls back to an advanced calculator mode (powered by `qalc`).
* **Status Bar Replacement:**
    * **Time & Calendar:** Current time, date, and a handy drop-down calendar.
    * **Battery:** Detailed stats (Status, Charge %, Consumption in Watts, Time until empty/full).
    * **Notifications:** Quick toggles for "Do Not Disturb" and clearing all notifications (via `dunstctl` by default).
* **Keyboard Navigation:** Full arrow key control, `Tab` / `Shift+Tab` support, and closing via `Esc` or clicking outside the window.

## Dependencies

To build and run Launchpad properly, you will need:
* **Rust / Cargo** (for building)
* **GTK4** and **gtk4-layer-shell** (for rendering the UI as an overlay)
* **qalc** (`libqalculate` package) for the built-in calculator engine.
* **dunst** (optional, command can be swapped out) for notification management.

## Installation

Building and installing the project is automated via the `Makefile`.

1. Clone the repository:
   ```bash
   git clone https://github.com/tebryaev/launchpad
   cd launchpad
   ```
2. Build and install system-wide:
   ```bash
   make install
   ```
   *This command builds the release binary and copies it to `/usr/bin/launchpad`.*

If you ever want to remove the application, simply run:
```bash
make uninstall
```

## Window Manager Integration

Launchpad is designed to be triggered by a hotkey. Here is how you can bind it in popular Wayland compositors:

**Hyprland** (`~/.config/hypr/hyprland.conf`):
```hyprlang
bind = $mainMod, Space, exec, launchpad
```

**Sway** (`~/.config/sway/config`):
```text
bindsym $mod+space exec launchpad
```

**Niri** (`~/.config/niri/config.kdl`):
```kdl
binds {
    Mod+Space { spawn "launchpad"; }
}
```


## Configuration

The tool is highly flexible. By default, the launcher uses the Nord color palette because that's what I personally use.

You can override all settings and styles locally:
1. Configuration: `~/.config/launchpad/config.toml`
2. Styles (CSS): `~/.config/launchpad/style.css`

You can grab the default `config.toml` and `style.css` directly from the root of this repository and copy them to your local `~/.config/launchpad/` directory.

### Example `config.toml`
You can change the application search paths, hide specific apps, change your battery device name, and override the commands used for notifications and the calculator:

```toml
cache_file = "~/.cache/launchpad.cache"

# Ignored applications (matched by the "Name" field in the .desktop file)
ignored_apps = [
    "xterm",
    "uxterm",
]

# Paths where the launcher looks for .desktop files
app_search_paths = [
    "/usr/share/applications",
    "~/.local/share/applications",
    "/var/lib/flatpak/exports/share/applications"
]

[battery]
# Device name in /sys/class/power_supply/
device = "BAT0"

[notifications]
# Commands for managing notifications (using dunst as an example).
# Passed as arrays: ["command", "argument1", "argument2"]
status_cmd = ["dunstctl", "is-paused"]
mute_cmd = ["dunstctl", "set-paused", "true"]
unmute_cmd = ["dunstctl", "set-paused", "false"]
clear_all_cmd = ["dunstctl", "close-all"]

[calculator]
# Command for calculations. The result is expected in stdout.
# '--' is used so qalc doesn't interpret the expression as its own flags.
command = ["qalc", "-t", "--"]
```

## Contributing
Right now, the project is strictly in the "works perfectly for me" stage. But if there's actual demand from the community, I'm completely open to improving the installation process, expanding the feature set, and adding more configuration flexibility. Pull requests and issues are highly welcome!