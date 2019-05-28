use core::ops::Deref;

use spin::Mutex;

use generic_array::GenericArray;
use generic_array::typenum::consts::U36;

use sunrise_libuser::error::Error;
use sunrise_libuser::ipc::*;

use sunrise_libuser::time::CalendarTime;
use sunrise_libuser::time::CalendarAdditionalInfo;
use sunrise_libuser::time::PosixTime;

use sunrise_libtimezone::TimeZoneRule;

use sunrise_libutils::initialize_to_zero;

/// Type representing a LocationName.
/// Used to get arround Default requirement of the IPC layer
type LocationNameInternal = GenericArray<u8, U36>;

type IpcResult<T> = Result<T, Error>;


// TODO: Move to FileSystem interface after implementation
include!(concat!(env!("OUT_DIR"), "/timezone_data.rs"));

/// Represent the file I/O interface with tzdata.
struct TimeZoneFileSystem;

/// Represent a file inside a TimeZoneFileSystem.
struct TimeZoneFile {
    data: &'static [u8]
}

impl TimeZoneFile {
    /// Create a TimeZoneFile instance from a raw slice.
    pub fn from_raw(data: &'static [u8]) -> Self {
        TimeZoneFile {
            data
        }
    }

    // Read the whole file.
    pub fn read_full(&self) -> &[u8] {
        self.data
    }
}

impl TimeZoneFileSystem {
    pub fn open_file(path: &[u8]) -> Option<TimeZoneFile> {
        for (file_path, data) in TIMEZONE_ARCHIVE.iter() {
            if *file_path == path {
                return Some(TimeZoneFile::from_raw(data))
            }
        }

        None
    }
}


/// Global instance handling I/O and storage of the device rules.
#[derive(Default)]
pub struct TimeZoneManager {
    location: LocationNameInternal,
    /// Rules of this device.
    my_rules: TimeZoneRule
}

impl TimeZoneManager {
    pub fn get_device_location_name(&self) -> LocationNameInternal {
        self.location
    }

    pub fn set_device_location_name(&mut self, location: LocationNameInternal) -> IpcResult<()> {
        // TODO: check names
        self.set_device_location_name_unchecked(location);
        Ok(())
    }

    pub fn set_device_location_name_unchecked(&mut self, location: LocationNameInternal) {
        self.location = location;
    }

    pub fn get_total_location_name_count(&self) -> IpcResult<(u32, )> {
        unimplemented!()
    }
}

// https://data.iana.org/time-zones/tzdata-latest.tar.gz

/// Global instance of TimeZoneManager
pub static TZ_MANAGER: Mutex<TimeZoneManager> = Mutex::new(unsafe { initialize_to_zero!(TimeZoneManager) });

/// TimeZone service object.
#[derive(Default, Debug)]
pub struct TimeZoneService {
    pub unknown: u64
}

impl Drop for TimeZoneService {
    fn drop(&mut self) {
        info!("DROP TZ");
    }
}

fn calendar_to_tzlib(ipc_calendar: &CalendarTime) -> sunrise_libtimezone::CalendarTimeInfo {
    let mut res = sunrise_libtimezone::CalendarTimeInfo::default();

    res.year = ipc_calendar.year as i64;
    res.month = ipc_calendar.month;
    res.day = ipc_calendar.day;
    res.hour = ipc_calendar.hour;
    res.minute = ipc_calendar.minute;
    res.second = ipc_calendar.second;

    res
}

fn calendar_to_ipc(tzlib_calendar: sunrise_libtimezone::CalendarTime) -> (CalendarTime, CalendarAdditionalInfo) {
    let calendar_time = CalendarTime {
        year: tzlib_calendar.time.year as i16,
        month: tzlib_calendar.time.month,
        day: tzlib_calendar.time.day,
        hour: tzlib_calendar.time.hour,
        minute: tzlib_calendar.time.minute,
        second: tzlib_calendar.time.second,
    };

    let additional_info = CalendarAdditionalInfo {
        day_of_week: tzlib_calendar.additional_info.day_of_week,
        day_of_year: tzlib_calendar.additional_info.day_of_year,
        tz_name: tzlib_calendar.additional_info.timezone_name,
        is_daylight_saving_time: tzlib_calendar.additional_info.is_dst,
        gmt_offset: tzlib_calendar.additional_info.gmt_offset,
    };

    (calendar_time, additional_info)
}

object! {
    impl TimeZoneService {
        #[cmdid(0)]
        #[inline(never)]
        fn get_device_location_name(&mut self, ) -> Result<(LocationNameInternal, ), Error> {
            let res = TZ_MANAGER.lock().get_device_location_name();
            Ok((res, ))
        }

        #[cmdid(1)]
        #[inline(never)]
        fn set_device_location_name(&mut self, location: LocationNameInternal,) -> Result<(), Error> {
            TZ_MANAGER.lock().set_device_location_name(location)
        }

        #[cmdid(2)]
        #[inline(never)]
        fn get_total_location_name_count(&mut self,) -> Result<(u32, ), Error> {
            TZ_MANAGER.lock().get_total_location_name_count()
        }

        #[cmdid(4)]
        #[inline(never)]
        fn load_timezone_rule(&mut self, location: LocationNameInternal, tz_rules: OutBuffer<TimeZoneRule>, ) -> Result<(), Error> {
            let mut tz_rules = tz_rules;
            *tz_rules = TimeZoneRule::default();
            Ok(())
        }

        #[cmdid(5)]
        #[inline(never)]
        fn test(&mut self, test: OutBuffer<LocationNameInternal>, ) -> Result<(), Error> {
            let mut test = test;

            test[0] = b'A';
            Ok(())
        }

        #[cmdid(100)]
        #[inline(never)]
        fn to_calendar_time(&mut self, time: PosixTime, timezone_buffer: InBuffer<TimeZoneRule>, ) -> Result<(CalendarTime, CalendarAdditionalInfo, ), Error> {
            let res = timezone_buffer.deref().to_calendar_time(time);
            if res.is_err() {
                // TODO: error managment here
                panic!()
            }

            let (calendar_time, calendar_additional_data) = calendar_to_ipc(res.unwrap());

            Ok((calendar_time, calendar_additional_data, ))
        }

        #[cmdid(200)]
        #[inline(never)]
        fn to_posix_time(&mut self, calendar_time: CalendarTime, timezone_buffer: InBuffer<TimeZoneRule>, ) -> Result<(PosixTime, ), Error> {
            let res = timezone_buffer.deref().to_posix_time(&calendar_to_tzlib(&calendar_time));
            if res.is_err() {
                // TODO: error managment here
                panic!()
            }

            Ok((res.unwrap(), ))
        }
    }
}