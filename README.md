# wib - Wayland Idle Blocker

wib prevents your user session from idling on Wayland, stopping screen blanking, locking, etc.  
Easily toggle the idle inhibition on/off using a signal.

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