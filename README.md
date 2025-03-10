# wib - Wayland Idle Blocker

wib prevents your user session from idling by disabling screen blanking, locking, and other idle behaviors - essentially providing a “presentation mode".
  
It's usable with Wayland compositors implementing the *layer-shell* protocol.  
You can toggle the idle inhibition on/off using `SIGUSR1`. The current state ("activated" or "deactivated") is printed so that you can easily integrate it with scripts or other tools.

## Usage

1. Simply start the application:
    ```bash
    wib
    ```
2. Toggle idle inhibition:
    ```bash
    pkill -USR1 wib
    ```
3. The inhibitor state is printed to standard output:
    ```bash
    activated
    ```

## Install
```bash
cargo install --git https://github.com/stefur/wib wib
```