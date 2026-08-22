use notat_core::{
    db::{
        devices::{delete_device, get_device_by_id, list_devices, touch_device, upsert_device},
        migrations::open_in_memory,
    },
    models::device::Device,
};

#[test]
fn test_device_lifecycle() {
    let conn = open_in_memory().unwrap();

    let device_1 = Device::new("dev_pixel", "Pixel 9", "mobile");
    let device_2 = Device::new("dev_mac", "MacBook Pro", "desktop");

    upsert_device(&conn, &device_1).unwrap();
    upsert_device(&conn, &device_2).unwrap();

    // Fetch by ID
    let fetched = get_device_by_id(&conn, "dev_pixel").unwrap().unwrap();
    assert_eq!(fetched.name, "Pixel 9");
    assert_eq!(fetched.platform, "mobile");

    // Touch device (bump last_seen_at)
    let new_timestamp = fetched.last_seen_at + 100_000;
    touch_device(&conn, "dev_pixel", new_timestamp).unwrap();

    let touched = get_device_by_id(&conn, "dev_pixel").unwrap().unwrap();
    assert_eq!(touched.last_seen_at, new_timestamp);

    // List devices (dev_pixel was touched last, should be first)
    let devices = list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].id, "dev_pixel");
    assert_eq!(devices[1].id, "dev_mac");

    // Delete device
    delete_device(&conn, "dev_pixel").unwrap();
    assert!(get_device_by_id(&conn, "dev_pixel").unwrap().is_none());
    assert_eq!(list_devices(&conn).unwrap().len(), 1);
}
