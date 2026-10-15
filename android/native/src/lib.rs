//! JNI boundary for the Android proof-of-concept.
//! All file permissions and reads are handled by Android's Storage Access Framework.
#![deny(clippy::all)]

use c2l_detection::{Verdict, classify};
use c2l_hashing::Sha256State;
use c2l_indicators::IndicatorDb;
use jni::JNIEnv;
use jni::objects::{JByteArray, JObject, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jlong, jstring};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};

const MAX_CHUNK_SIZE: usize = 64 * 1024;
const MAX_SESSIONS: usize = 8;
static NEXT_ID: AtomicI64 = AtomicI64::new(1);
static SESSIONS: OnceLock<Mutex<HashMap<i64, Sha256State>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<i64, Sha256State>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn start_session() -> i64 {
    let Ok(mut sessions) = sessions().lock() else {
        return 0;
    };

    if sessions.len() >= MAX_SESSIONS {
        return 0;
    }

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    if id <= 0 {
        return 0;
    }

    sessions.insert(id, Sha256State::new());
    id
}

fn update_session(id: i64, bytes: &[u8]) -> bool {
    if bytes.len() > MAX_CHUNK_SIZE {
        return false;
    }

    let Ok(mut sessions) = sessions().lock() else {
        return false;
    };

    match sessions.get_mut(&id) {
        Some(session) => {
            session.update(bytes);
            true
        }
        None => false,
    }
}

fn finish_session(id: i64) -> Option<String> {
    let mut sessions = sessions().lock().ok()?;
    sessions.remove(&id).map(Sha256State::finish)
}

fn cancel_session(id: i64) {
    if let Ok(mut sessions) = sessions().lock() {
        sessions.remove(&id);
    }
}

fn to_java_string(env: &mut JNIEnv<'_>, value: &str) -> jstring {
    env.new_string(value)
        .map(JString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_c2l_antivirus_RustEngine_startHash(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
) -> jlong {
    start_session()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_c2l_antivirus_RustEngine_updateHash(
    env: JNIEnv<'_>,
    _this: JObject<'_>,
    id: jlong,
    bytes: JByteArray<'_>,
    count: jint,
) -> jboolean {
    if count < 0 || count as usize > MAX_CHUNK_SIZE {
        return JNI_FALSE;
    }

    let Ok(data) = env.convert_byte_array(&bytes) else {
        return JNI_FALSE;
    };

    if count as usize > data.len() {
        return JNI_FALSE;
    }

    if update_session(id, &data[..count as usize]) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_c2l_antivirus_RustEngine_finishHash(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
    id: jlong,
) -> jstring {
    match finish_session(id) {
        Some(hash) => to_java_string(&mut env, &hash),
        None => std::ptr::null_mut(),
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_c2l_antivirus_RustEngine_cancelHash(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
    id: jlong,
) {
    cancel_session(id);
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_c2l_antivirus_RustEngine_classifyHash(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
    hash: JString<'_>,
    indicators: JString<'_>,
) -> jstring {
    let Ok(hash) = env.get_string(&hash) else {
        return std::ptr::null_mut();
    };
    let hash: String = hash.into();

    let Ok(indicators) = env.get_string(&indicators) else {
        return std::ptr::null_mut();
    };
    let indicators: String = indicators.into();

    let verdict = match IndicatorDb::parse(Cursor::new(indicators.as_bytes())) {
        Err(error) => format!("INDICATOR DATABASE ERROR: {error}"),
        Ok(database) => match classify(&hash, Some(&database)) {
            Verdict::NotChecked => "NOT CHECKED".to_string(),
            Verdict::NotListed => "NOT LISTED (not a safety guarantee)".to_string(),
            Verdict::Listed(name) => format!("MATCH ({name})"),
        },
    };

    to_java_string(&mut env, &verdict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_native_session_matches_sha256() {
        let id = start_session();
        assert!(id > 0);
        assert!(update_session(id, b"a"));
        assert!(update_session(id, b"bc"));
        assert_eq!(
            finish_session(id).as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn canceled_session_cannot_be_used() {
        let id = start_session();
        cancel_session(id);
        assert!(!update_session(id, b"abc"));
        assert!(finish_session(id).is_none());
    }
}
