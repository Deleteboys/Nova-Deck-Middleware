use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
}

pub fn calculate_next_position(start: i32, step: i32) -> i32 {
    if step == 0 {
        return start.clamp(0, 100);
    }

    let is_even = step % 2 == 0;
    let grid = if is_even { 5 } else { 10 };
    let org_next = start + step;

    if is_even && start % 10 == 5 {
        let delta = if step > 0 { 1 } else { -1 };
        return (start + delta).clamp(0, 100);
    }

    let crossed_boundary = (start / grid) != (org_next / grid);
    let is_on_boundary = (start % grid) == 0;

    let target = if crossed_boundary && !is_on_boundary {
        if step > 0 {
            ((start / grid) + 1) * grid
        } else {
            (start / grid) * grid
        }
    } else {
        org_next
    };

    target.clamp(0, 100)
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::*;