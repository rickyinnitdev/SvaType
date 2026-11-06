use khmer_core::{
    cluster,
    layout::{Layout, PreparedLayout},
    plan,
    session::{KeyEvent, Policy, Session},
};
use wasm_bindgen::prelude::*;

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("engine values serialize")
}
fn error(value: impl ToString) -> JsValue {
    JsValue::from_str(&value.to_string())
}

#[wasm_bindgen]
pub fn segment(text: &str) -> String {
    let spans = cluster::segment(text);
    json(&spans.iter().map(|c|serde_json::json!({"start":c.start,"end":c.end,"char_start":c.char_start,"char_end":c.char_end,"text":&text[c.start..c.end]})).collect::<Vec<_>>())
}

#[wasm_bindgen(js_name=Layout)]
pub struct WasmLayout {
    inner: PreparedLayout,
}
#[wasm_bindgen(js_class=Layout)]
impl WasmLayout {
    #[wasm_bindgen(js_name=from_json)]
    pub fn from_json(source: &str) -> Result<WasmLayout, JsValue> {
        let layout: Layout = serde_json::from_str(source).map_err(error)?;
        Ok(Self {
            inner: layout.prepare().map_err(error)?,
        })
    }
    pub fn plan(&self, text: &str) -> Result<String, JsValue> {
        plan::plan(text, &self.inner)
            .map(|p| json(&p))
            .map_err(|e| error(json(&e)))
    }
    pub fn validate_target(&self, text: &str) -> Result<(), JsValue> {
        plan::validate_target(text, &self.inner).map_err(|e| error(json(&e)))
    }
}

#[wasm_bindgen(js_name=Session)]
pub struct WasmSession {
    inner: Session,
}
#[wasm_bindgen(js_class=Session)]
impl WasmSession {
    pub fn observe_time(&mut self, timestamp_ms: f64) {
        if timestamp_ms.is_finite() && timestamp_ms >= 0.0 {
            self.inner.observe_time(timestamp_ms as u64);
        }
    }
    pub fn cursor(&self) -> u32 {
        self.inner.cluster_index() as u32
    }
    pub fn free_typing(layout: &WasmLayout) -> WasmSession {
        Self {
            inner: Session::free_typing(layout.inner.clone()),
        }
    }
    pub fn next_round(&mut self) {
        self.inner.next_round();
    }
    pub fn live(&self) -> String {
        json(&self.inner.live())
    }
    pub fn tail(&self) -> String {
        self.inner.tail(300)
    }
    pub fn checkpoint(&self) -> String {
        json(&self.inner.checkpoint())
    }
    pub fn restore(source: &str, layout: &WasmLayout) -> Result<WasmSession, JsValue> {
        let snapshot = serde_json::from_str(source).map_err(error)?;
        Ok(Self {
            inner: Session::restore(snapshot, layout.inner.clone()).map_err(|e| error(json(&e)))?,
        })
    }
    #[wasm_bindgen(constructor)]
    pub fn new(text: &str, layout: &WasmLayout, flow: bool, strict: bool) -> Result<Self, JsValue> {
        Ok(Self {
            inner: Session::new(
                text,
                layout.inner.clone(),
                if flow {
                    Policy::Flow
                } else {
                    Policy::StopOnError
                },
                strict,
            )
            .map_err(|e| error(json(&e)))?,
        })
    }
    #[wasm_bindgen(js_name=on_key)]
    #[allow(clippy::too_many_arguments)]
    pub fn on_key(
        &mut self,
        code: &str,
        shift: bool,
        altgr: bool,
        ctrl: bool,
        meta: bool,
        repeat: bool,
        timestamp_ms: f64,
    ) -> String {
        let timestamp_ms = if timestamp_ms.is_finite() && timestamp_ms >= 0.0 {
            timestamp_ms as u64
        } else {
            0
        };
        json(&self.inner.on_key(KeyEvent {
            code,
            shift,
            altgr,
            ctrl,
            meta,
            repeat,
            timestamp_ms,
        }))
    }
    pub fn summary(&self) -> String {
        json(&self.inner.summary())
    }
    pub fn expected(&self) -> String {
        json(&self.inner.expected_next())
    }
    pub fn typed(&self) -> String {
        self.inner.typed()
    }
    pub fn restart(&mut self) {
        self.inner.restart();
    }
}
#[wasm_bindgen]
pub fn prepare_target(text: &str) -> String {
    plan::prepare_target(text)
}
