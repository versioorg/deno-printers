use std::ffi::{CStr, CString};

use deno_bindgen::deno_bindgen;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::ptr;

// #[derive(Serialize)]
pub struct PrinterWrapper<'a> {
    pub printer: &'a printers::printer::Printer,
}

fn printer_state_to_string(state: &printers::printer::PrinterState) -> &'static str {
    match state {
        printers::printer::PrinterState::READY => "READY",
        printers::printer::PrinterState::PAUSED => "PAUSED",
        printers::printer::PrinterState::PRINTING => "PRINTING",
        printers::printer::PrinterState::UNKNOWN => "UNKNOWN",
    }
}

impl<'a> Serialize for PrinterWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state_value = printer_state_to_string(&self.printer.state);
        let mut printer_json = serializer.serialize_struct("Printer", 9)?;
        printer_json.serialize_field("name", &self.printer.name)?;
        printer_json.serialize_field("system_name", &self.printer.system_name)?;
        printer_json.serialize_field("driver_name", &self.printer.driver_name)?;
        printer_json.serialize_field("uri", &self.printer.uri)?;
        printer_json.serialize_field("location", &self.printer.location)?;
        printer_json.serialize_field("is_default", &self.printer.is_default)?;
        printer_json.serialize_field("is_shared", &self.printer.is_shared)?;
        printer_json.serialize_field("state", &state_value)?;
        printer_json.end()
    }
}

#[deno_bindgen]
fn get_printer_by_name(name: *mut i8) -> *mut i8 {
    if name.is_null() {
        eprintln!("Error: Printer name is null");
        return ptr::null_mut();
    }

    let cstr_name = unsafe { CStr::from_ptr(name) };
    let printer_name = match cstr_name.to_str() {
        Ok(n) => {
            println!("Looking up printer by name: {}", n);
            n
        },
        Err(_) => {
            eprintln!("Error: Invalid UTF-8 sequence in printer name");
            return ptr::null_mut();
        },
    };

    match printers::get_printer_by_name(printer_name) {
        Some(printer) => {
            println!("Printer found: {}", printer.name);
            let printer_wrapper = PrinterWrapper { printer: &printer };
            match serde_json::to_string(&printer_wrapper) {
                Ok(json_str) => CString::new(json_str).unwrap().into_raw(),
                Err(err) => {
                    eprintln!("Error serializing printer to JSON: {}", err);
                    ptr::null_mut()
                },
        }
        }
        None => {
            eprintln!("Error: Printer not found");
            ptr::null_mut()
        }
    }
}

#[deno_bindgen]
fn get_printers() -> *mut i8 {
    println!("Retrieving all printers...");
    let printers = printers::get_printers();
    let printer_wrappers = printers
        .iter()
        .map(|printer| PrinterWrapper { printer: &printer })
        .collect::<Vec<PrinterWrapper>>();

    match serde_json::to_string(&printer_wrappers) {
        Ok(json_str) => {
            println!("Successfully retrieved printers, serializing to JSON");
            CString::new(json_str).unwrap().into_raw()
        },
        Err(err) => {
            eprintln!("Error serializing printers to JSON: {}", err);
            ptr::null_mut()
        }
    }
}

#[deno_bindgen]
fn print(printer: *mut i8, text: *mut i8, job_name: *mut i8) -> bool {
    unsafe {
        if printer.is_null() {
            eprintln!("Error: Printer name is null");
            return false;
        }
        if text.is_null() {
            eprintln!("Error: Text to print is null");
            return false;
        }
        if job_name.is_null() {
            eprintln!("Error: Job name is null");
            return false;
        }

        let cstr_printer = CStr::from_ptr(printer);
        let printer_name = match cstr_printer.to_str() {
            Ok(n) => {
                println!("Looking up printer: {}", n);
                n
            },
            Err(_) => {
                eprintln!("Error: Invalid UTF-8 sequence in printer name");
                return false;
            },
        };

        let printer = match printers::get_printer_by_name(printer_name) {
            Some(p) => p,
            None => {
                eprintln!("Error: Printer not found");
                return false;
            }
        };

        let text = match CStr::from_ptr(text).to_str() {
            Ok(t) => t.as_bytes(),
            Err(_) => {
                eprintln!("Error: Invalid UTF-8 sequence in text");
                return false;
            }
        };

        let job_name = match CStr::from_ptr(job_name).to_str() {
            Ok(n) => Some(n),
            Err(_) => {
                eprintln!("Error: Invalid UTF-8 sequence in job name");
                return false;
            }
        };

        match printer.print(text, job_name) {
            Ok(_) => {
                println!("Print job '{}' sent to printer '{}'", job_name.unwrap_or("Unnamed"), printer_name);
                true
            },
            Err(err) => {
                eprintln!("Error printing: {}", err);
                false
            }
        }
    }
}

#[deno_bindgen]
fn print_file(printer: *mut i8, file: *mut i8, job_name: *mut i8) -> bool {
    unsafe {
        if printer.is_null() || file.is_null() || job_name.is_null() {
            eprintln!("Error: One or more arguments are null");
            return false;
        }

        let cstr_printer = CStr::from_ptr(printer);
        let printer_name = match cstr_printer.to_str() {
            Ok(n) => {
                println!("Looking up printer: {}", n);
                n
            },
            Err(_) => {
                eprintln!("Error: Invalid UTF-8 sequence in printer name");
                return false;
            }
        };

        let printer = match printers::get_printer_by_name(printer_name) {
            Some(p) => p,
            None => {
                eprintln!("Error: Printer not found");
                return false;
            }
        };

        let file = match CStr::from_ptr(file).to_str() {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Error: Invalid UTF-8 sequence in file path");
                return false;
            }
        };

        let job_name = match CStr::from_ptr(job_name).to_str() {
            Ok(n) => Some(n),
            Err(_) => {
                eprintln!("Error: Invalid UTF-8 sequence in job name");
                return false;
            }
        };

        match printer.print_file(file, job_name) {
            Ok(_) => {
                println!("File '{}' sent to printer '{}'", file, printer_name);
                true
            },
            Err(err) => {
                eprintln!("Error printing file: {}", err);
                false
            }
        }
    }
}
