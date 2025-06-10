use crate::models::models::pds as models;
use anyhow::Result;
use diesel::*;
use diesel::{
    delete, insert_into, update, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use rsky_common;
use rsky_oauth::oauth_provider::device::device_data::DeviceData;
use rsky_oauth::oauth_provider::device::device_id::DeviceId;
use rsky_oauth::oauth_provider::device::device_store::PartialDeviceData;
use rsky_oauth::oauth_provider::device::session_id::SessionId;

fn row_to_device_data(device: models::Device) -> DeviceData {
    DeviceData {
        user_agent: device.user_agent,
        ip_address: device.ip_address.parse().unwrap(),
        session_id: SessionId::new(device.session_id.unwrap()).unwrap(),
        last_seen_at: device.last_seen_at,
    }
}

pub async fn create_device(
    device_id: DeviceId,
    data: DeviceData,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<()> {
    use crate::schema::pds::device::dsl as DeviceSchema;
    db.get()
        .await
        .expect("Failed to get DB connection")
        .interact(move |conn| {
            let rows: Vec<models::Device> = vec![models::Device {
                id: device_id.into_inner(),
                session_id: Some(data.session_id.into_inner()),
                user_agent: data.user_agent,
                ip_address: data.ip_address.to_string(),
                last_seen_at: data.last_seen_at,
            }];
            insert_into(DeviceSchema::device)
                .values(&rows)
                .execute(conn)
        })
        .await
        .expect("Failed to create device")?;
    Ok(())
}

pub async fn read_device(
    device_id: DeviceId,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<Option<DeviceData>> {
    use crate::schema::pds::device::dsl as DeviceSchema;

    let device_id = device_id.into_inner();
    let result = db
        .get()
        .await
        .expect("Failed to get DB connection")
        .interact(move |conn| {
            DeviceSchema::device
                .filter(DeviceSchema::id.eq(device_id))
                .select(models::Device::as_select())
                .first(conn)
                .optional()
        })
        .await
        .expect("Failed to read device")?;
    Ok(result.map(row_to_device_data))
}

pub async fn update_device(
    device_id: DeviceId,
    opts: PartialDeviceData,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<()> {
    use crate::schema::pds::device::dsl as DeviceSchema;
    // db.get()
    //     .await
    //     .expect("Failed to get DB connection")
    //     .interact(move |conn| {
    //         //TODO: Implement update logic as needed
    //         // See original comments for details
    //         Ok(())
    //     })
    //     .await
    //     .expect("Failed to update device")?;
    Ok(())
}

pub async fn delete_device(
    device_id: DeviceId,
    db: &deadpool_diesel::Pool<
        deadpool_diesel::Manager<SqliteConnection>,
        deadpool_diesel::sqlite::Object,
    >,
) -> Result<()> {
    use crate::schema::pds::device::dsl as DeviceSchema;

    let device_id = device_id.into_inner();
    db.get()
        .await
        .expect("Failed to get DB connection")
        .interact(move |conn| {
            delete(DeviceSchema::device)
                .filter(DeviceSchema::id.eq(device_id))
                .execute(conn)
        })
        .await
        .expect("Failed to delete device")?;
    Ok(())
}

pub struct UpdateDeviceOpt {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub session_id: Option<SessionId>,
    pub last_seen_at: Option<u64>,
}
