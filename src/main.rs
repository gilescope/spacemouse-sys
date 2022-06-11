//#![feature(core_c_str)]
#![feature(untagged_unions)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
extern crate libc;
//use core::ffi::CStr;
//use libc::c_char;
//use std::ffi::c_void;
//use std::ffi::CString;
//const fn aa(x: usize) {}

// There are two plugin operating modes: one takes over the device
// and disables all built-in driver assignments, the other complements
// the driver by only executing commands that are meant for plugins:

// enum {
// 	kConnexionClientModeTakeOver		= 1,		// take over device completely, driver no longer executes assignments
// 	kConnexionClientModePlugin			= 2			// receive plugin assignments only, let driver take care of its own
// };
const kConnexionClientModeTakeOver: u16 = 1;
const kConnexionClientModePlugin: u16 = 2;

const kConnexionMaskAll: u32 = 0x3FFF;

const kConnexionMaskAllButtons: u32 = 0xFFFFFFFF;

const kConnexionDeviceStateType: u32 = 0x4D53; // 'MS' (Connexion State)
const kConnexionDeviceStateVers: u32 = 0x6D33; // 'm3' (version 3 includes 32-bit button data in previously unused field, binary compatible with version 2)

//#define kConnexionCtlSetSwitches		'3dss'		// set the current state of the client-controlled feature switches (bitmap, see masks below)
// = 862221171 as u32

#[repr(C)]
pub union ConnexionMsg {
    state: ConnexionDeviceState,
    version: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct ConnexionDeviceState {
    // header
    version: u16, // kConnexionDeviceStateVers
    client: u16, // identifier of the target client when sending a state message to all user clients
    // command
    command: u16, // command for the user-space client
    param: i16,   // optional parameter for the specified command
    value: i32,   // optional value for the specified command
    time: u64,    // timestamp for this message (clock_get_uptime)
    // raw report
    report: [u8; 8], // raw USB report from the device
    // processed data
    buttons8: u16, // buttons (first 8 buttons only, for backwards binary compatibility- use "buttons" field instead)
    axis: [i16; 6], // x, y, z, rx, ry, rz
    address: u16,  // USB device address, used to tell one device from the other
    buttons: u32,  // buttons
}

#[repr(C)]
#[derive(Debug)]
struct ConnexionDevicePrefs {
    // header
    pref_type: u16, // kConnexionDevicePrefsType
    version: u16,   // kConnexionDevicePrefsVers
    deviceID: u16,  // device ID (SpaceNavigator, SpaceNavigatorNB, SpaceExplorer...)
    reserved1: u16, // set to 0
    // target application
    appSignature: u32, // target application signature
    reserved2: u32,    // set to 0
    appName: [u8; 64], // target application name (Pascal string with length byte at the beginning)
    // device preferences
    mainSpeed: u8,        // overall speed
    zoomOnY: u8,          // use Y axis for zoom, Z axis for un/down pan
    dominant: u8,         // only respond to the largest one of all 6 axes values at any given time
    reserved3: u8,        // set to 0
    mapV: [i8; 6],        // axes mapping when Zoom direction is on vertical axis (zoomOnY = 0)
    mapH: [i8; 6],        // axes mapping when Zoom direction is on horizontal axis (zoomOnY != 0)
    enabled: [u8; 6],     // enable or disable individual axes
    reversed: [u8; 6],    // reverse individual axes
    speed: [u8; 6],       // speed for individual axes (min 0, max 200, reserved 201-255)
    sensitivity: [u8; 6], // sensitivity for individual axes (min 0, max 200, reserved 201-255)
    scale: [i32; 6],      // 10000 * scale and "natural" reverse state for individual axes
    // added in version 10.0 (build 136)
    gamma: u32, // 1000 * gamma value used to compute nonlinear axis response, use 1000 (1.0) for linear response
    intersect: u32, // intersect value used for gamma computations
}

#[no_mangle]
pub extern "C" fn message_handler(product_id: u32, msg_type: u32, msg: *mut ConnexionMsg) {
    println!("got msg from {}", product_id);
    match msg_type {
        kConnexionDeviceStateType => {
            unsafe {
                let msg: &ConnexionDeviceState = &((*msg).state);
                // if client id are not same you're not meant to read this msg.
                println!("state {:?}", msg);
            }
        }
        kConnexionDeviceStateVers => {
            // Don't touch msg, it's an int!

            unsafe {
                let msg: u32 = (*msg).version;
                println!("ver {}", msg);
            }
        }
        _ => {
            eprintln!("well that was a surprise")
        }
    }
}

#[no_mangle]
pub extern "C" fn added(product_id: u32) {
    println!("added {}", product_id);
}

#[no_mangle]
pub extern "C" fn removed(product_id: u32) {
    println!("removed {}", product_id);
}

const EXE_NAME: &[u8; 9] = b"spacemac\0";

const NULL: &[u8; 1] = b"\0";

fn main() {
    // int16_t			SetConnexionHandlers				(ConnexionMessageHandlerProc messageHandler,
    //ConnexionAddedHandlerProc addedHandler, ConnexionRemovedHandlerProc removedHandler,
    //bool useSeparateThread);

    let client_id: u16;
    unsafe {
        let set_res = SetConnexionHandlers(
            message_handler, // as *const fn(usize, usize, *mut String),
            added,           // as *const fn(usize),
            removed,         // as *const fn(usize),
            true,
        );
        
        println!(
            "SetConnexionHandlers res {} should probably return 0",
            set_res
        );

        client_id = RegisterConnexionClient(
            0,
            //NULL as *const u8, 
            EXE_NAME as *const u8, // Any exe has focus grab the input still
            kConnexionClientModePlugin, //kConnexionClientModeTakeOver,
            kConnexionMaskAll,
        );
        if client_id == 0 {
            eprintln!("bad client id");
        }

        println!("RegisterConnexionClient client_id {}", client_id);

        SetConnexionClientMask(client_id, kConnexionMaskAll);
        SetConnexionClientButtonMask(client_id, kConnexionMaskAllButtons);

        //set(AxisMode: AxisMode.AllAxis, On: true)

        // let prefs: ConnexionDevicePrefs = ConnexionDevicePrefs{
        //     pref_type: (), version: (), deviceID: (), reserved1: (),
        //      appSignature: (), reserved2: (), appName: (), mainSpeed: (),
        //       zoomOnY: (), dominant: (), reserved3: (), mapV: (), mapH: (),
        //        enabled: (), reversed: (), speed: (), sensitivity: (), scale: (),
        //         gamma: (), intersect: () };

        // let res = ConnexionGetCurrentDevicePrefs(client_id, &prefs);
        // println!("res {}", res);

    //     	func openPreferencesPane() -> Void{
	// 	ConnexionClientControl(ClientId, ConnexionClient.Ctrl.OpenPrefPane, 0, nil)
	// }
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("fin");
    unsafe {
        UnregisterConnexionClient(client_id);
    }
}

#[cfg(target_os = "macos")]
#[link(name = "3DconnexionClient", kind = "framework")]
extern "C" {
    fn SetConnexionHandlers(
        message_handler: extern "C" fn(u32, u32, *mut ConnexionMsg) -> (),
        connection_added: extern "C" fn(u32) -> (),
        connection_removed: extern "C" fn(u32) -> (),
        separare_thread: bool,
    ) -> u16;

    fn RegisterConnexionClient(signature: u16, name: *const u8, mode: u16, mask: u32) -> u16; //client_id

    fn UnregisterConnexionClient(client_id: u16);

    fn SetConnexionClientMask(client_id: u16, mask: u32);

    fn SetConnexionClientButtonMask(client_id: u16, button_mask: u32);

    fn ConnexionGetCurrentDevicePrefs(productID: u32, prefs: *mut ConnexionDevicePrefs) -> u16;

}
