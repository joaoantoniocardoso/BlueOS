use serde::Serialize;

use crate::id::JourneyId;
use crate::sitl_cal::{SitlRc, SITL_FRAME_CALIBRATION, SITL_FRAME_VECTORED};

pub const UI_CALIBRATION_JOURNEYS: &[JourneyId] = &[
    JourneyId::CalibrateGyroscope,
    JourneyId::CalibrateBarometer,
    JourneyId::LevelHorizon,
    JourneyId::CalibrateAccelerometer,
    JourneyId::CalibrateCompass,
    JourneyId::DetectMotorDirections,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiJourneyPlan {
    pub journey_id: String,
    pub sitl_frame: Option<&'static str>,
    pub actions: Vec<UiAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum UiAction {
    Open {
        path: &'static str,
    },
    Expect {
        text: &'static str,
    },
    Click {
        text: &'static str,
    },
    ClickIfVisible {
        text: &'static str,
    },
    WaitText {
        text: &'static str,
        timeout_ms: u64,
    },
    SitlRc {
        chan5: u16,
        chan6: u16,
        chan7: u16,
        chan8: u16,
    },
    Sleep {
        ms: u64,
    },
}

pub fn wizard_skip_plan() -> UiJourneyPlan {
    UiJourneyPlan {
        journey_id: "wizard_skip".into(),
        sitl_frame: None,
        actions: vec![UiAction::Open { path: "/" }],
    }
}

pub fn ui_plan(id: JourneyId) -> Option<UiJourneyPlan> {
    let actions = match id {
        JourneyId::CalibrateGyroscope => vec![
            open_configure("gyroscope"),
            sitl_rc(SitlRc::stop()),
            UiAction::Sleep { ms: 1500 },
            UiAction::Click {
                text: "Calibrate Gyroscopes",
            },
            UiAction::WaitText {
                text: "Calibration done.",
                timeout_ms: 20_000,
            },
        ],
        JourneyId::CalibrateBarometer => vec![
            open_configure("baro"),
            sitl_rc(SitlRc::stop()),
            UiAction::Sleep { ms: 1500 },
            UiAction::Expect {
                text: "Calibrate Barometer",
            },
            UiAction::Click { text: "Calibrate" },
            UiAction::WaitText {
                text: "Calibration done.",
                timeout_ms: 20_000,
            },
        ],
        JourneyId::LevelHorizon => vec![
            open_configure("accelerometer"),
            sitl_rc(SitlRc::attitude_deg(0, 0, 0)),
            UiAction::Sleep { ms: 2000 },
            UiAction::Click {
                text: "Level Horizon",
            },
            UiAction::Sleep { ms: 500 },
            UiAction::Click { text: "Calibrate" },
            UiAction::WaitText {
                text: "Calibration finished",
                timeout_ms: 20_000,
            },
        ],
        JourneyId::CalibrateAccelerometer => accel_plan(),
        JourneyId::CalibrateCompass => vec![
            open_configure("compass"),
            sitl_rc(SitlRc::mag_dance()),
            UiAction::Click {
                text: "Full (Onboard) Calibration",
            },
            UiAction::Click {
                text: "Start Full Calibration",
            },
            UiAction::Sleep { ms: 2000 },
            UiAction::ClickIfVisible {
                text: "Use GeoIP coordinates",
            },
            UiAction::Click { text: "Calibrate" },
            UiAction::WaitText {
                text: "Dismiss",
                timeout_ms: 180_000,
            },
        ],
        JourneyId::DetectMotorDirections => vec![
            UiAction::Open {
                path: "/vehicle/setup/pwm_outputs",
            },
            UiAction::Click {
                text: "Detect Reversed Motors",
            },
            UiAction::Click {
                text: "Start Detection",
            },
            UiAction::WaitText {
                text: "Motor direction detection is complete",
                timeout_ms: 120_000,
            },
        ],
        _ => return None,
    };
    Some(UiJourneyPlan {
        journey_id: id.as_str().to_string(),
        sitl_frame: if matches!(id, JourneyId::DetectMotorDirections) {
            Some(SITL_FRAME_VECTORED)
        } else {
            Some(SITL_FRAME_CALIBRATION)
        },
        actions,
    })
}

pub fn ui_suite_plans() -> Vec<UiJourneyPlan> {
    UI_CALIBRATION_JOURNEYS
        .iter()
        .copied()
        .filter_map(ui_plan)
        .collect()
}

fn open_configure(subtab: &'static str) -> UiAction {
    match subtab {
        "gyroscope" => UiAction::Open {
            path: "/vehicle/setup/configure/gyroscope",
        },
        "baro" => UiAction::Open {
            path: "/vehicle/setup/configure/baro",
        },
        "accelerometer" => UiAction::Open {
            path: "/vehicle/setup/configure/accelerometer",
        },
        "compass" => UiAction::Open {
            path: "/vehicle/setup/configure/compass",
        },
        _ => UiAction::Open {
            path: "/vehicle/setup/configure",
        },
    }
}

fn sitl_rc(rc: SitlRc) -> UiAction {
    UiAction::SitlRc {
        chan5: rc.chan5,
        chan6: rc.chan6,
        chan7: rc.chan7,
        chan8: rc.chan8,
    }
}

fn accel_plan() -> Vec<UiAction> {
    let poses: [(&str, i16, i16, i16); 6] = [
        ("Place the vehicle on a level surface", 0, 0, 0),
        ("Place the vehicle on its left side", -90, 0, 0),
        ("Place the vehicle on its right side", 90, 0, 0),
        ("Place the vehicle with its nose down", 0, 90, 0),
        ("Place the vehicle with its nose up", 0, -90, 0),
        ("Place the vehicle on its back", 0, 180, 0),
    ];
    let mut actions = vec![
        open_configure("accelerometer"),
        sitl_rc(SitlRc::attitude_deg(0, 0, 0)),
        UiAction::Sleep { ms: 2500 },
        UiAction::Click {
            text: "Start Full Calibration",
        },
        UiAction::Click {
            text: "Start Calibration",
        },
    ];
    for (i, (prompt, roll, pitch, yaw)) in poses.iter().enumerate() {
        if i > 0 {
            actions.push(sitl_rc(SitlRc::attitude_deg(*roll, *pitch, *yaw)));
        }
        actions.push(UiAction::WaitText {
            text: prompt,
            timeout_ms: 30_000,
        });
        actions.push(UiAction::Sleep { ms: 1500 });
        actions.push(UiAction::Click { text: "Next" });
        actions.push(UiAction::Sleep { ms: 2000 });
    }
    actions.push(UiAction::WaitText {
        text: "Calibrated",
        timeout_ms: 30_000,
    });
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_journeys_all_have_plans() {
        for id in UI_CALIBRATION_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(!plan.actions.is_empty());
            assert!(plan.sitl_frame.is_some());
        }
    }

    #[test]
    fn gyro_plan_opens_configure_and_stops_sitl() {
        let plan = ui_plan(JourneyId::CalibrateGyroscope).unwrap();
        assert!(matches!(
            &plan.actions[0],
            UiAction::Open {
                path: "/vehicle/setup/configure/gyroscope"
            }
        ));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::SitlRc { .. })));
        assert_eq!(plan.sitl_frame, Some(SITL_FRAME_CALIBRATION));
    }

    #[test]
    fn accel_plan_has_six_attitudes() {
        let plan = ui_plan(JourneyId::CalibrateAccelerometer).unwrap();
        let attitudes = plan
            .actions
            .iter()
            .filter(
                |a| matches!(a, UiAction::SitlRc { chan5, .. } if *chan5 >= 1100 && *chan5 < 1200),
            )
            .count();
        assert_eq!(attitudes, 6);
    }

    #[test]
    fn motor_detect_uses_vectored() {
        let plan = ui_plan(JourneyId::DetectMotorDirections).unwrap();
        assert_eq!(plan.sitl_frame, Some(SITL_FRAME_VECTORED));
    }

    #[test]
    fn compass_plan_tries_geoip_if_present() {
        let plan = ui_plan(JourneyId::CalibrateCompass).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickIfVisible {
                text: "Use GeoIP coordinates"
            }
        )));
    }

    #[test]
    fn unknown_journey_has_no_plan() {
        assert!(ui_plan(JourneyId::RenameVehicle).is_none());
    }
}
