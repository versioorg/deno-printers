/**
* This module contains functions for interacting with hardware printers.
* @module
*/

import { dlopen } from "jsr:@denosaurs/plug@1.0.6";
import type { Printer } from "./types.ts";
import metadata from "./deno.json" with { type: "json" };

const { symbols } = await dlopen(
  {
    name: "printer_bindings",
    url: `https://github.com/versioorg/deno-printers/releases/download/${metadata.version}/`,
  },
  {
    get_printer_by_name: {
      parameters: ["buffer"],
      result: "pointer",
      nonblocking: false,
    },
    get_printers: {
      parameters: [],
      result: "pointer",
      nonblocking: false,
    },
    print_file: {
      parameters: ["buffer", "buffer", "buffer"],
      result: "bool",
      nonblocking: false,
    },
    print: {
      parameters: ["buffer", "buffer", "buffer"],
      result: "bool",
      nonblocking: false,
    },
    print_pdf_file: { // ✅ Dodajemy nowy symbol
      parameters: ["buffer", "buffer", "buffer"],
      result: "bool",
      nonblocking: false,
    },
  }
);

/** 
 * This function returns a list of printers connected to the system. 
 * @returns A list of printers. {@link Printer}
 */
export function getPrinters(): Printer[] {
  const pointer = symbols.get_printers();
  return JSON.parse(new Deno.UnsafePointerView(pointer!).getCString());
}

/** 
 * This function returns a printer by name. 
 * @param name The name of the printer to return.
 * @returns The printer with the given name.
 */
export function getPrinterByName(name: string): Printer {
  const pointer = symbols.get_printer_by_name(new TextEncoder().encode(name + "\0"));
  return JSON.parse(new Deno.UnsafePointerView(pointer!).getCString());
}

/** 
 * This function prints a string to a printer. 
 * @param printer The printer to print to.
 * @param text The text to print.
 * @param jobName The name of the job.  
 * @returns True if the string was printed successfully, false otherwise.
 */
export function print(printer: Printer, text: string, jobName?: string): boolean {
  const encoder = new TextEncoder();
  return symbols.print(
    encoder.encode(printer.name + "\0"),
    encoder.encode(text + "\0"),
    jobName ? encoder.encode(jobName + "\0") : null,
  );
}

/** 
 * This function prints a file by a given path to a printer. 
 * @param printer The printer to print to.
 * @param file The path to the file to print.
 * @param jobName The name of the job.  
 * @returns True if the file was printed successfully, false otherwise.
 */
export function printFile(printer: Printer, file: string, jobName?: string): boolean {
  const encoder = new TextEncoder();
  return symbols.print_file(
    encoder.encode(printer.name + "\0"),
    encoder.encode(file + "\0"),
    jobName ? encoder.encode(jobName + "\0") : null,
  );
}

/** 
 * This function prints a PDF file using a system-aware method (e.g. lp or PowerShell).  
 * @param printer The printer to print to.
 * @param file The path to the PDF file.
 * @param jobName The name of the job.  
 * @returns True if the PDF was printed successfully, false otherwise.
 */
export function printPdfFile(printer: Printer, file: string, jobName?: string): boolean {
  const encoder = new TextEncoder();
  return symbols.print_pdf_file(
    encoder.encode(printer.name + "\0"),
    encoder.encode(file + "\0"),
    jobName ? encoder.encode(jobName + "\0") : null,
  );
}
