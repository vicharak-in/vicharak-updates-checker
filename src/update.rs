use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use whoami::username;

const GIT_URL: &str = "http://github.com/vicharak-in/";
// GIT_URL/vicharak-PKGBUILDS
const PKGBUILD_URL: &str = "https://raw.githubusercontent.com/vicharak-in/vicharak-PKGBUILDS/";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OSType {
    Arch,
    Debian,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub version: (u8, u8, u8),
    pub os: OSType,
}

#[derive(Serialize, Deserialize, Default)]
pub struct VicharakPackages {
    pub packages: Vec<Package>,
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}@{}.{}.{}",
            self.name, self.version.0, self.version.1, self.version.2
        )?;
        Ok(())
    }
}

impl std::fmt::Display for VicharakPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();
        for package in &self.packages {
            output.push_str(&format!(
                "{}: {}.{}.{}, ",
                package.name, package.version.0, package.version.1, package.version.2
            ));
        }
        write!(f, "{output}")
    }
}

pub fn read_vicharak_packages() -> Result<VicharakPackages, std::io::Error> {
    // check if file exists
    let path = format!("/home/{}/.vicharak_packages.json", username());
    if !std::path::Path::new(&path).exists() {
        let mut file = File::create(&path)?;
        file.write_all(
            serde_json::to_string_pretty(&VicharakPackages::default())
                .unwrap()
                .as_bytes(),
        )?;

        return Ok(VicharakPackages::default());
    }

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let packages: VicharakPackages = serde_json::from_str(&contents)?;
    Ok(packages)
}

pub enum PacmanWrapper {
    Pacman,
    Yay,
    Paru,
}

pub fn get_pacman_wrapper() -> Result<PacmanWrapper, std::io::Error> {
    if Path::new("/usr/bin/yay").exists() {
        Ok(PacmanWrapper::Yay)
    } else if Path::new("/usr/bin/paru").exists() {
        Ok(PacmanWrapper::Paru)
    } else {
        Ok(PacmanWrapper::Pacman)
    }
}

impl Package {
    pub fn new(name: String, version: (u8, u8, u8)) -> Package {
        Package {
            name,
            version,
            os: Package::get_host_os_type(),
        }
    }

    pub fn get_host_os_type() -> OSType {
        if Path::new("/usr/bin/pacman").exists() {
            OSType::Arch
        } else if Path::new("/usr/bin/apt").exists() {
            OSType::Debian
        } else {
            OSType::Unknown
        }
    }

    /// Get latest version of a package from the server (http://35.187.91.110/vicharak/).
    pub async fn get_latest_version_arch(
        name: &str,
    ) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
        let url = format!("{PKGBUILD_URL}{name}/master/PKGBUILD");
        let text = reqwest::get(&url).await?.text().await?;

        // get pkgver: (x.x.x) from PKGBUILD
        let pkgver = text
            .split("pkgver=")
            .nth(1)
            .expect("Could not find pkgver in PKGBUILD")
            .split_whitespace()
            .next()
            .unwrap()
            .trim_matches('"')
            .get(..5)
            .expect("Malformed pkgver in PKGBUILD")
            .split('.')
            .map(|v| v.parse::<u8>().expect("Malformed pkgver in PKGBUILD"))
            .collect::<Vec<u8>>();

        Ok((pkgver[0], pkgver[1], pkgver[2]))
    }

    pub async fn get_latest_version_debian(
        name: &str,
    ) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
        let url = format!("{}{}/raw/master/debian/changelog", GIT_URL, name);

        println!("Getting latest version for {}", name);
        let text = reqwest::get(url).await?.text().await?;

        // get pkg version from debian/changelog
        let pkgver = text
            .split('(')
            .nth(1)
            .expect("Could not find version in debian/changelog")
            .split(')')
            .next()
            .unwrap()
            .split('.')
            .map(|v| {
                v.parse::<u8>()
                    .expect("Malformed version in debian/changelog")
            })
            .collect::<Vec<u8>>();

        println!("Latest version for {} is {}.{}.{}", name, pkgver[0], pkgver[1], pkgver[2]);

        Ok((pkgver[0], pkgver[1], pkgver[2]))
    }

    pub fn get_current_version(name: &str) -> std::io::Result<Option<(u8, u8, u8)>> {
        let packages = read_vicharak_packages()?;

        // check if packages is empty
        if packages.packages.is_empty() {
            println!("No packages found");
            return Ok(None);
        }

        if let Some(package) = packages.packages.iter().find(|p| p.name == name) {
            return Ok(Some(package.version));
        }

        Ok(None)
    }

    pub fn check_update(&self) -> std::io::Result<bool> {
        let current_version = Package::get_current_version(&self.name)?;
        if let Some(current_version) = current_version {
            if current_version.0 < self.version.0
                || current_version.1 < self.version.1
                || current_version.2 < self.version.2
            {
                println!(
                    "{}: {}.{}.{} -> {}.{}.{}",
                    self.name,
                    current_version.0,
                    current_version.1,
                    current_version.2,
                    self.version.0,
                    self.version.1,
                    self.version.2
                );
                return Ok(true);
            } else if current_version.0 == self.version.0
                && current_version.1 == self.version.1
                && current_version.2 == self.version.2
            {
                println!("{}: up to date", self.name);
                return Ok(false);
            } else {
                println!(
                    "Downgrade({}: {}.{}.{} -> {}.{}.{})",
                    self.name,
                    current_version.0,
                    current_version.1,
                    current_version.2,
                    self.version.0,
                    self.version.1,
                    self.version.2
                );
                return Ok(false);
            }
        }

        Ok(false)
    }

    fn pacman_command(&self) -> Result<Command, std::io::Error> {
        let mut command = Command::new("sudo");

        match get_pacman_wrapper()? {
            PacmanWrapper::Pacman => {
                command.arg("pacman");
            }
            PacmanWrapper::Yay => {
                command.arg("yay");
            }
            PacmanWrapper::Paru => {
                command.arg("paru");
            }
        }

        command.arg("-S").arg("--noconfirm").arg(&self.name);

        Ok(command)
    }

    fn apt_command(&self) -> Result<Command, std::io::Error> {
        let mut command = Command::new("sudo");

        command.arg("apt").arg("install").arg("-y").arg(&self.name);

        Ok(command)
    }

    pub fn update_package(&self) -> Result<(), std::io::Error> {
        match self.os {
            OSType::Arch => {
                self.pacman_command()?.spawn()?.wait()?;
            }

            OSType::Debian => {
                self.apt_command()?.spawn()?.wait()?;
            }

            OSType::Unknown => {
                println!("Unknown OS");
            }
        }
        Ok(())
    }
}

impl VicharakPackages {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    pub fn add_package(&mut self, package: Package) {
        self.packages.push(package);
    }

    pub fn save_packages_to_file(&self) -> std::io::Result<()> {
        for package in &self.packages {
            let mut vicharak_packages = read_vicharak_packages()?;

            // check if package already exists
            if Package::get_current_version(&package.name)?.is_none() {
                vicharak_packages.add_package(package.clone());
            }

            let path = format!("/home/{}/.vicharak_packages.json", username());
            let json_string = serde_json::to_string_pretty(&vicharak_packages)?;
            let mut file = File::create(&path)?;
            file.write_all(json_string.as_bytes())?;
        }

        Ok(())
    }

    pub fn check_updates(&self) -> std::io::Result<Vec<Package>> {
        let mut updates = Vec::new();

        println!("Checking for updates: {self}");
        for package in &self.packages {
            if package.check_update()? {
                updates.push(package.clone());
            }
        }

        Ok(updates)
    }
}
