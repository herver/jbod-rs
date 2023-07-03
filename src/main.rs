/*-
 * SPDX-License-Identifier: BSD-2-Clause
 *
 * BSD 2-Clause License
 *
 * Copyright (c) 2021-2023, Gandi S.A.S.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#[forbid(unsafe_code)]
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::*;
use jbod::enclosure::BackPlane::create_enclosure_table;
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use std::process::{exit, Command};

#[macro_use] extern crate prettytable;
use prettytable::format;
use prettytable::{Table, Cell, Row};

mod jbod;
mod utils;
use crate::jbod::disks::DiskShelf;
use crate::jbod::enclosure::BackPlane;
use crate::utils::helper::Util;

/// Fallback help function, we should never fall here
fn help() {
    println!("Use command with help option");
}

/// Given a string representing a temperature like: [0-9]+ it will
/// return colored string first for the temperature second for the unit.
///
/// Coloration:
///
/// - Bellow 50 it's all green
/// - Between 45 excluded and below 50 included it's yellow bold
/// - Above it's blinking red you must act maybe :)
///
/// If temperature is not readable it return `None` it's caller responsibility
/// to report it properly.
///
fn color_temp(temperature: &str) -> Option<(ColoredString, ColoredString)> {
    let temp_conv = temperature.parse::<i32>().ok()?;
    let coloreds = if temp_conv > 45 && temp_conv <= 50 {
        (temperature.yellow().bold(),
        "c".yellow().bold())
    } else if temp_conv > 50 {
        (temperature.red().bold().blink(), "c".red().bold().blink())
    } else {
        (temperature.green(), "c".green())
    };
    Some(coloreds)
}

/// TODO: Rework error handling, perhaps we don't need return Result
///
/// Returns an empty Result for now.
///
/// This function is used in the `list` menu option,
/// it combines the options for list the enclosures,
/// disks and fan from the JBOD.
///
/// # Arguments
///
/// * `option` - clappy's ArgMatches
///
fn enclosure_overview(option: &ArgMatches) -> Result<(), ()> {
    let disks_option = option.is_present("disks");
    let enclosure_option = option.is_present("enclosure");
    let fan_option = option.is_present("fan");
    let temperature_option = option.is_present("temperature");
    let voltage_option = option.is_present("voltage");

    // If the options `-ed` or `-d` are used, it shows
    // the enclosure and disks altogether.
    if enclosure_option && disks_option || disks_option {
        let enclosure = BackPlane::get_enclosure();
        let mut disks = DiskShelf::jbod_disk_map();
        disks.sort_by_key(|d| d.slot.clone());


        for enc in enclosure {
            print!("{}", enc);     

            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            table.set_titles(row!["Disk", "Map", "Slot", "Vendor", "Model", "Serial", "Temp", "Fw"]);
            for disk in &disks {
                if enc.slot == disk.enclosure {
                    let mut row: Vec<Cell> = Vec::new();
                    row.push(Cell::new(&disk.device_path).style_spec("Fg"));
                    // row.push(Cell::new(&disk.device_path).style_spec("Fg"));
                    if disk.device_map == "NONE" {
                        row.push(Cell::new(&disk.device_map).style_spec("Fy"));
                    } else {
                        row.push(Cell::new(&disk.device_map).style_spec("Fg"));
                    }
                    row.push(Cell::new(&disk.slot.as_str()).style_spec("Fg"));
                    row.push(Cell::new(&disk.vendor).style_spec("Fb"));
                    row.push(Cell::new(&disk.model).style_spec("Fb"));
                    row.push(Cell::new(&disk.serial).style_spec("Fg"));
                    match color_temp(&disk.temperature) {
                        Some((temp_colored, unit_colored)) => row.push(Cell::new(format!("{}{:<2}", temp_colored, unit_colored).as_str())),
                        None => row.push(Cell::new("ERR").style_spec("bFR")),
                    }

                    row.push(Cell::new(&disk.fw_revision).style_spec("Fb"));
                    table.add_row(Row::new(row));
                }
            }
            table.printstd();
        }
    // Here it shows only the enclosures.
    } else if enclosure_option && !disks_option {
        let enclosure = BackPlane::get_enclosure();

        // Prepare table
        let mut enc_table = create_enclosure_table();
        for enc in enclosure {
            let mut enc_row: Vec<Cell> = Vec::new();
            enc_row.push(Cell::new(&enc.slot));
            enc_row.push(Cell::new(&enc.device_path));
            enc_row.push(Cell::new(&enc.vendor));
            enc_row.push(Cell::new(&enc.model));
            enc_row.push(Cell::new(&enc.revision));
            enc_row.push(Cell::new(&enc.serial));
            enc_table.add_row(Row::new(enc_row));
        }
        enc_table.printstd();

    // Here it shows the FAN.
    } else if fan_option {
        let enclosure_fan = BackPlane::get_enclosure_fan();
        let mut fan_table = BackPlane::create_fan_table();
        fan_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        for fan in enclosure_fan {
            fan_table.add_row(Row::new(vec![
                Cell::new(&fan.slot),
                Cell::new(&fan.index),
                Cell::new(&fan.description),
                Cell::new(&fan.comment),
                Cell::new(&fan.speed.to_string()),
            ]));
        }
        fan_table.printstd();
    } else if temperature_option {
        let enclosure_temp = BackPlane::get_enclosure_temp();
        let mut temp_table = BackPlane::create_temp_table();
        temp_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        for temp in enclosure_temp {
            if temp.status != "Not installed" {
                temp_table.add_row(Row::new(vec![
                    Cell::new(&temp.slot),
                    Cell::new(&temp.index),
                    Cell::new(&temp.description),
                    Cell::new(&temp.status),
                    Cell::new(&temp.temperature.to_string()),
                ]));
            }
        }
        temp_table.printstd();
    } else if voltage_option {
        let enclosure_voltage = BackPlane::get_enclosure_voltage();
        let mut voltage_table = BackPlane::create_voltage_table();
        voltage_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        for voltage in enclosure_voltage {
            if voltage.status != "Not installed" {
                voltage_table.add_row(Row::new(vec![
                    Cell::new(&voltage.slot),
                    Cell::new(&voltage.index),
                    Cell::new(&voltage.description),
                    Cell::new(&voltage.status),
                    Cell::new(&voltage.voltage.to_string()),
                ]));
            }
        }
        voltage_table.printstd();
    }

    Ok(())
}

/// TODO: Rework error handling, perhaps we don't need return Result 
///
/// Returns an empty Result for now.
///
/// This function forks another binary for the prometheus-exporter. 
///
/// # Arguments
///
/// * `option` - clappy's ArgMatches
///
fn fork_prometheus(option: &ArgMatches) -> Result<(), ()> {
    let mut default_port = "9945";
    let mut default_address = "0.0.0.0";

    if let Some(port) = option.value_of("port") {
        default_port = port;
    }

    if let Some(ip) = option.value_of("ip-address") {
        default_address = ip;
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("prometheus-exporter pid: {:?}", child);
            waitpid(Some(child), None).unwrap();
            exit(0);
        }

        Ok(ForkResult::Child) => {
            Command::new(Util::JBOD_EXPORTER)
                .args(&[default_address, default_port])
                .spawn()
                .expect("Failed to spawn the target process");
            exit(0);
        }
        Err(_) => println!("Fork Failed"),
    }

    Ok(())
}

/// The main function that creates the menu.
fn main() {
    Util::verify_binary_needed();

    let matches = App::new("jbod")
        .version("0.0.1")
        .author("\nAuthor: Marcelo Araujo <marcelo.araujo@gandi.net>")
        .about("About: A generic storage enclosure tool")
        .subcommand(
            SubCommand::with_name("list")
                .about("list")
                .arg_required_else_help(true)
                .arg(
                    Arg::with_name("enclosure")
                        .short('e')
                        .long("enclosure")
                        .multiple(false)
                        .required(false)
                        .takes_value(false)
                        .help("List enclosure"),
                )
                .arg(
                    Arg::with_name("disks")
                        .short('d')
                        .long("disks")
                        .multiple(false)
                        .required(false)
                        .takes_value(false)
                        .help("List disks"),
                )
                .arg(
                    Arg::with_name("fan")
                        .short('f')
                        .long("fan")
                        .multiple(false)
                        .required(false)
                        .takes_value(false)
                        .exclusive(false)
                        .help("List fan"),
                )
                .arg(
                    Arg::with_name("voltage")
                        .short('v')
                        .long("voltage")
                        .multiple(false)
                        .required(false)
                        .takes_value(false)
                        .exclusive(false)
                        .help("List voltage sensors"),
                )
                .arg(
                    Arg::with_name("temperature")
                        .short('t')
                        .long("temperature")
                        .multiple(false)
                        .required(false)
                        .takes_value(false)
                        .exclusive(false)
                        .help("List temperature sensors"),
                 ),
        )
        .subcommand(
            SubCommand::with_name("led")
                .about("led")
                .arg(
                    Arg::with_name("locate")
                        .short('l')
                        .long("locate")
                        .required(false)
                        .multiple(true)
                        .value_name("DEVICE")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("fault")
                        .short('f')
                        .long("fault")
                        .required(false)
                        .multiple(true)
                        .value_name("DEVICE")
                        .takes_value(true),
                )
                .arg(Arg::with_name("on").long("on").required(false))
                .arg(Arg::with_name("off").long("off").required(false)),
        )
        .subcommand(
            SubCommand::with_name("prometheus")
                .about("Prometheus")
                .arg(
                    Arg::with_name("port")
                        .short('p')
                        .long("port")
                        .required(false)
                        .value_name("PORT")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ip-address")
                        .alias("ip")
                        .long("ip-address")
                        .required(false)
                        .value_name("IPADDRESS")
                        .takes_value(true),
                ),
        )
        .get_matches();

    // Here it matches the menu options with its respective functions.
    match matches.subcommand() {
        Some(("list", m)) => enclosure_overview(m),
        Some(("led", m)) => DiskShelf::jbod_led_switch(m),
        Some(("prometheus", m)) => fork_prometheus(m),
        _ => Ok(help()),
    };
}
