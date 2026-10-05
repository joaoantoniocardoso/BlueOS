use std::sync::Arc;

use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, SetLevelGoal},
    std_msgs::Empty,
};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

use super::tank::{
    FragileLevel, FragileTankService, LEVEL_THAT_FAILS_TO_ENCODE, LEVEL_THAT_PANICS_IN_HANDLE,
    MisnamedTankService, NotAPercent, ProbeArguments, ProbeService, Tank, TankArguments, TankQuery,
    TankRequest, TankResponse, TankSnapshot,
};

impl Service for FragileTankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "fragile_tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(
            ServiceBuilder::new(TankSnapshot::empty(service.arguments().capacity))
                .command("SetLevel", |request: SetLevelGoal| {
                    Ok(TankRequest::SetLevel(request.level))
                })
                .query(
                    "Level",
                    |_request: Empty| Ok(TankQuery::Level),
                    |response: TankResponse| match response {
                        TankResponse::Level(level) => Some(FragileLevel { level }),
                        TankResponse::Other => None,
                    },
                )
                .state("tank", |snapshot: &TankSnapshot| FragileLevel {
                    level: snapshot.level,
                }),
        )
    }
}

impl Service for ProbeService {
    type Domain = Tank;
    type Context = ();
    type Arguments = ProbeArguments;

    const NAME: &'static str = "probe";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<ProbeArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<ProbeArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(ServiceBuilder::new(TankSnapshot::empty(100))
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(TankRequest::SetLevel(request.level))
            })
            .io_query("Probe", {
                let sensor = Arc::clone(&service.arguments().sensor.0);
                move |request: SetLevelGoal| {
                    let sensor = Arc::clone(&sensor);
                    Box::pin(async move {
                        sensor.acquire().await?.forget();
                        assert_ne!(request.level, LEVEL_THAT_PANICS_IN_HANDLE);
                        if request.level > 100 {
                            return Err(NotAPercent(request.level).into());
                        }
                        Ok(LevelResponse {
                            level: request.level,
                            max_level: 100,
                        })
                    })
                }
            })
            .io_query("FragileProbe", |_request: Empty| {
                Box::pin(async {
                    Ok(FragileLevel {
                        level: LEVEL_THAT_FAILS_TO_ENCODE,
                    })
                })
            }))
    }
}

impl Service for MisnamedTankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "misnamed_tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(
            ServiceBuilder::new(TankSnapshot::empty(service.arguments().capacity))
                .command("Set#Level", |request: SetLevelGoal| {
                    Ok(TankRequest::SetLevel(request.level))
                }),
        )
    }
}
