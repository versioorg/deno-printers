use std::ffi::{CStr, CString};

use deno_bindgen::deno_bindgen;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::{fs, ptr};

//use std::process::Command;
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

        match my_print(printer_name, file, job_name) {
            Ok(_) => {
                println!("Document sent to printer successfully.");
                true
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                false
            }
        }
    }
}


pub fn my_print(printer_system_name: &str, file_path: &str, job_name: Option<&str>) -> Result<bool, String> {
    // Jeśli job_name nie jest przekazany, użyj file_path jako domyślną nazwę zadania
    let job_name = job_name.unwrap_or(file_path);

    
        // Odczyt pliku do stringa
        let file_content = match fs::read(file_path) {
            Ok(bytes) => bytes,
            Err(e) => return Err(format!("Failed to read file '{}': {}", file_path, e)),
        };

        // Wywołanie funkcji write_to_device, aby wysłać zawartość pliku do drukarki
        match write_to_device(printer_system_name, &file_content) {
            Ok(bytes_written) => {
                println!(
                    "Successfully sent {} bytes to the printer '{}' for job '{}'.",
                    bytes_written, printer_system_name, job_name
                );
                Ok(true)
            }
            Err(e) => Err(format!(
                "Failed to write to the printer '{}': {}",
                printer_system_name, e
            )),
        }
}

// Funkcja write_to_device wysyłająca dane do urządzenia drukarki na Windowsie
// from https://crates.io/crates/raw-printer
#[cfg(target_os = "windows")]
pub fn write_to_device(printer: &str, payload: &[u8]) -> Result<usize, std::io::Error> {
    use std::ffi::CString;
    use std::ptr;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Graphics::Printing::{
        ClosePrinter, EndDocPrinter, EndPagePrinter, OpenPrinterA, StartDocPrinterA,
        StartPagePrinter, WritePrinter, DOC_INFO_1A, PRINTER_ACCESS_USE, PRINTER_DEFAULTSA,
    };

    let printer_name = CString::new(printer).unwrap_or_default(); // null-terminated string
    let mut printer_handle: HANDLE = HANDLE(std::ptr::null_mut());

    // Otwórz drukarkę
    unsafe {
        let pd = PRINTER_DEFAULTSA {
            pDatatype: windows::core::PSTR(ptr::null_mut()),
            pDevMode: ptr::null_mut(),
            DesiredAccess: PRINTER_ACCESS_USE,
        };

        if OpenPrinterA(
            windows::core::PCSTR(printer_name.as_bytes().as_ptr()),
            &mut printer_handle,
            Some(&pd),
        )
        .is_ok()
        {
            let doc_info = DOC_INFO_1A {
                pDocName: windows::core::PSTR("My Document\0".as_ptr() as *mut u8),
                pOutputFile: windows::core::PSTR::null(),
                pDatatype: windows::core::PSTR("RAW\0".as_ptr() as *mut u8),
            };

            // Rozpocznij dokument
            let job = StartDocPrinterA(printer_handle, 1, &doc_info as *const _ as _);
            if job == 0 {
                return Err(std::io::Error::from(windows::core::Error::from_win32()));
            }

            // Rozpocznij stronę
            if !StartPagePrinter(printer_handle).as_bool() {
                return Err(std::io::Error::from(windows::core::Error::from_win32()));
            }

            let buffer = payload;

            let mut bytes_written: u32 = 0;
            if !WritePrinter(
                printer_handle,
                buffer.as_ptr() as _,
                buffer.len() as u32,
                &mut bytes_written,
            )
            .as_bool()
            {
                return Err(std::io::Error::from(windows::core::Error::from_win32()));
            }

            // Zakończ stronę i dokument
            let _ = EndPagePrinter(printer_handle);
            let _ = EndDocPrinter(printer_handle);
            let _ = ClosePrinter(printer_handle);
            return Ok(bytes_written as usize);
        } else {
            return Err(std::io::Error::from(windows::core::Error::from_win32()));
        }
    }
}

#[cfg(target_os = "linux")]
pub fn write_to_device(printer: &str, payload: &[u8]) -> Result<usize, std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let device_path = OpenOptions::new().write(true).open(printer);

    match device_path {
        Ok(mut device) => {
            let bytes_written = device.write(payload)?;
            Ok(bytes_written)
        }
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}


//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////// wydruk PDF /////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[deno_bindgen]
fn print_pdf_file(printer: *mut i8, file: *mut i8, job_name: *mut i8) -> bool {
    unsafe {
        if printer.is_null() || file.is_null() {
            eprintln!("Error: One or more arguments are null");
            return false;
        }

        let cstr_printer = CStr::from_ptr(printer);
        let printer_name = match cstr_printer.to_str() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Invalid UTF-8 in printer name");
                return false;
            }
        };

        let cstr_file = CStr::from_ptr(file);
        let file_path = match cstr_file.to_str() {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Invalid UTF-8 in file path");
                return false;
            }
        };

        let job_name_str = if job_name.is_null() {
            None
        } else {
            Some(
                match CStr::from_ptr(job_name).to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("Invalid UTF-8 in job name");
                        return false;
                    }
                },
            )
        };

        match print_pdf(printer_name, file_path, job_name_str) {
            Ok(_) => {
                println!("PDF sent to printer successfully.");
                true
            }
            Err(e) => {
                eprintln!("Error printing PDF: {}", e);
                false
            }
        }
    }
}

pub fn print_pdf(printer: &str, file_path: &str, _job_name: Option<&str>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: użyj polecenia PowerShell do drukowania pliku PDF
        use std::process::Command;

        let printer_arg = format!("{}", printer);
        let mut command = Command::new("powershell");
        command.args(&[
            "-NoProfile",
            "-Command",
            &format!(
                r#"Start-Process -FilePath "{}" -Verb Print -ArgumentList '/p /h "{}"'"#,
                file_path, printer_arg
            ),
        ]);

        let output = command.output().map_err(|e| format!("Failed to run powershell: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Printing failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: użyj `lp` z podaniem drukarki
        use std::process::Command;

        let mut cmd = Command::new("lp");
        cmd.arg("-d").arg(printer).arg(file_path);

        if let Some(name) = _job_name {
            cmd.arg("-t").arg(name);
        }

        let output = cmd.output().map_err(|e| format!("Failed to run lp: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "lp command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}
