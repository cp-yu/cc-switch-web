use std::sync::Arc;

use cc_switch_core::CoreContext;

use crate::events::EventSender;

pub struct ServerState {
    pub auth_token: Option<String>,
    pub event_bus: EventSender,
    pub core: CoreContext,
}

impl ServerState {
    pub fn new(auth_token: Option<String>, event_bus: EventSender) -> Arc<Self> {
        // 初始化核心上下文（数据库、SkillService 等）
        let core = CoreContext::new().expect("failed to initialize cc-switch core context");
        Arc::new(Self {
            auth_token,
            event_bus,
            core,
        })
    }
}
