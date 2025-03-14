use crate::client::Client;
use nix::sys::memfd::{MemFdCreateFlag, memfd_create};
use nix::unistd::ftruncate;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM, SIGUSR1};
use signal_hook::iterator::Signals;
use std::error::Error;
use std::ffi::CString;
use std::os::fd::AsFd;
use wayland_client::Connection;
use wayland_client::protocol::wl_shm::Format;
use wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer::Overlay;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;

mod client;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let arg = args[1].as_str();
        if arg == "--help" || arg == "-h" {
            println!(
                r#"
wib - Wayland Idle Blocker
==========================

Description:
    wib prevents the user session from idling, such as screen blanking, locking, etc. 
    Run wib and send SIGUSR1 (e.g. via 'pkill -USR1 wib') to toggle the inhibitor on/off.

Options:
    -h, --help    Show this help message and exit
                "#
            );
        } else {
            println!("Unknown argument. Use --help or -h for more information.")
        }
        return Ok(());
    }

    let mut signals = Signals::new([
        SIGUSR1, // For toggling the inhibitor
        SIGTERM, // For quit
        SIGINT,  // For quit
        SIGQUIT, // Also for quit
    ])?;

    // Connect to the Wayland server
    let connection =
        Connection::connect_to_env().expect("Failed to connect to the Wayland server.");

    let display = connection.display();

    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();

    let _registry = display.get_registry(&queue_handle, ());

    let mut client = Client::new();

    event_queue.roundtrip(&mut client)?;

    // Create a surface
    let surface = client
        .compositor
        .as_ref()
        .expect("Failed to create surface. Is a compositor running?")
        .create_surface(&queue_handle, ());

    // Set up a layer surface
    let layer_surface = client
        .layer_shell
        .as_ref()
        .expect("Failed to create a layer surface.")
        .get_layer_surface(
            &surface,
            None,
            Overlay,
            String::from("wib"),
            &queue_handle,
            (),
        );

    // Set a minimal fixed size
    layer_surface.set_size(1, 1);

    // Set anchors
    layer_surface.set_anchor(Anchor::Top | Anchor::Left);

    surface.commit();
    event_queue.roundtrip(&mut client)?;

    // Create a buffer
    let buffer = {
        let width = 1;
        let height = 1;
        let stride = width * 4;
        let size = height * stride;
        let name = CString::new("wib").expect("Failed to create CString.");

        // Create a memfd
        let fd = memfd_create(&name, MemFdCreateFlag::empty())
            .expect("Failed to create file descriptor");

        // Truncate the file descriptor
        ftruncate(&fd, size as i64).expect("Truncating the file descriptor failed.");

        let pool = client
            .wl_shm
            .as_ref()
            .expect("wl_shm was None when attempting to create a pool.")
            .create_pool(fd.as_fd(), size, &queue_handle, ());
        let buffer = pool.create_buffer(
            0,
            width,
            height,
            stride,
            Format::Argb8888,
            &queue_handle,
            (),
        );
        pool.destroy();
        buffer
    };

    surface.attach(Some(&buffer), 0, 0);
    surface.commit();
    event_queue.roundtrip(&mut client)?;

    let mut inhibitor: Option<ZwpIdleInhibitorV1> = None;
    println!("deactivated");

    for signal in signals.forever() {
        match signal {
            SIGUSR1 => {
                if let Some(inhibitor) = inhibitor.take() {
                    inhibitor.destroy();
                    println!("deactivated");
                } else {
                    inhibitor = Some(
                        client
                            .inhibit_manager
                            .as_ref()
                            .expect("No idle inhibit manager found.")
                            .create_inhibitor(&surface, &queue_handle, ()),
                    );
                    println!("activated");
                }
            }
            SIGTERM | SIGINT | SIGQUIT => {
                break;
            }
            _ => unreachable!(),
        }

        event_queue.roundtrip(&mut client)?;
    }
    println!("quitting");
    // Cleanup: destroy the surface and any inhibitor before exiting
    if let Some(inhibitor) = inhibitor.take() {
        inhibitor.destroy();
    }
    layer_surface.destroy();
    surface.destroy();
    buffer.destroy();
    client.destroy_all();
    event_queue.roundtrip(&mut client)?;
    Ok(())
}
