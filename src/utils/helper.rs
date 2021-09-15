/*-
 * SPDX-License-Identifier: BSD-2-Clause
 *
 * BSD 2-Clause License
 *
 * Copyright (c) 2021, Gandi S.A.S.
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

#[allow(non_snake_case)]
pub mod Util {
    use colored::*;
    use std::path::Path;
    use std::process::exit;

    pub const LSSCSI: &str = "/usr/bin/lsscsi";
    pub const SG_INQ: &str = "/usr/bin/sg_inq";
    pub const SCSI_TEMP: &str = "/usr/bin/scsi_temperature";
    pub const SG_MAP: &str = "/usr/bin/sg_map";
    pub const SG_SES: &str = "/usr/bin/sg_ses";
    pub const SGINFO: &str = "/usr/bin/sginfo";
    pub const JBOD_EXPORTER: &str = "/usr/bin/prometheus-jbod-exporter";

    /// Returns true or false for a given path
    ///
    /// This function verify if a path exist.
    ///
    /// # Arguments
    ///
    /// * `path` - a string reference
    ///
    pub fn path_exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Verify if all needed binaries are installed
    pub fn verify_binary_needed() {
        let mut binaries_not_found = Vec::new();
        if !path_exists(LSSCSI) {
            binaries_not_found.push("lsscsi");
        }
        if !path_exists(SG_INQ) {
            binaries_not_found.push("sg3-utils");
        }
        if !path_exists(SCSI_TEMP) {
            binaries_not_found.push("sg3-utils: scsi_temperature");
        }

        if !binaries_not_found.is_empty() {
            println!(
                "{} {} {}",
                "==> ".blue().bold(),
                "Packages missing".bold(),
                " <==".blue().bold()
            );
            for err in binaries_not_found {
                print!("{}", ":: ".bold().red());
                print!("Install package ");
                println!("{}", err.red().bold().blink());
            }
            exit(1);
        }
    }

    /// Returns true or false for every each character
    ///
    /// This function verify is a string is numeric.
    ///
    /// # Arguments
    ///
    /// * `s` - a string
    pub fn is_string_numeric(s: String) -> bool {
        for c in s.chars() {
            if !c.is_numeric() {
                return false;
            }
        }
        return true;
    }
}
