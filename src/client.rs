use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_registry::{Event::Global, WlRegistry},
        wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1;
use wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Event as LayerSurfaceEvent;
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub struct Client {
    pub compositor: Option<WlCompositor>,
    pub inhibit_manager: Option<ZwpIdleInhibitManagerV1>,
    pub wl_shm: Option<WlShm>,
    pub layer_shell: Option<ZwlrLayerShellV1>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            compositor: None,
            inhibit_manager: None,
            wl_shm: None,
            layer_shell: None,
        }
    }
    pub fn destroy_all(&mut self) {
        if let Some(layer_shell) = self.layer_shell.take() {
            layer_shell.destroy();
        }
        if let Some(inhibit_manager) = self.inhibit_manager.take() {
            inhibit_manager.destroy();
        }
    }
}

impl Dispatch<WlRegistry, ()> for Client {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _: &(),
        _: &Connection,
        queue_handle: &QueueHandle<Self>,
    ) {
        if let Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == WlCompositor::interface().name && version >= 6 {
                state.compositor =
                    Some(registry.bind::<WlCompositor, _, Self>(name, 6, queue_handle, ()));
            }
            if interface == ZwpIdleInhibitManagerV1::interface().name && version >= 1 {
                state.inhibit_manager = Some(registry.bind::<ZwpIdleInhibitManagerV1, _, Self>(
                    name,
                    1,
                    queue_handle,
                    (),
                ));
            }
            if interface == ZwlrLayerShellV1::interface().name && version >= 4 {
                state.layer_shell =
                    Some(registry.bind::<ZwlrLayerShellV1, _, Self>(name, 4, queue_handle, ()));
            }
            if interface == WlShm::interface().name && version >= 2 {
                state.wl_shm = Some(registry.bind::<WlShm, _, Self>(name, 2, queue_handle, ()));
            }
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for Client {
    fn event(
        _: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: LayerSurfaceEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let LayerSurfaceEvent::Configure {
            width: _,
            height: _,
            serial,
        } = event
        {
            layer_surface.ack_configure(serial);
        }
    }
}

delegate_noop!(Client: ZwlrLayerShellV1);

delegate_noop!(Client: ZwpIdleInhibitManagerV1);
delegate_noop!(Client: ZwpIdleInhibitorV1);

delegate_noop!(Client: ignore WlSurface);
delegate_noop!(Client: WlCompositor);
delegate_noop!(Client: WlShmPool);
delegate_noop!(Client: ignore WlShm);
delegate_noop!(Client: ignore WlBuffer);
