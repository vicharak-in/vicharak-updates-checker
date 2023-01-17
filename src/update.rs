use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use whoami::username;

#[derive(Serialize, Deserialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: (u8, u8, u8),
}

#[derive(Serialize, Deserialize, Default)]
pub struct VaamanPackages {
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

impl std::fmt::Display for VaamanPackages {
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

pub fn read_vaaman_packages() -> Result<VaamanPackages, std::io::Error> {
    // check if file exists
    let path = format!("/home/{}/.vaaman_packages.json", username());
    if !std::path::Path::new(&path).exists() {
        let mut file = File::create(&path)?;
        file.write_all(
            serde_json::to_string(&VaamanPackages::default())
                .unwrap()
                .as_bytes(),
        )?;
    }

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let packages: VaamanPackages = serde_json::from_str(&contents)?;
    Ok(packages)
}

impl Package {
    pub fn new(name: String, version: (u8, u8, u8)) -> Package {
        Package { name, version }
    }

    /// Get latest version of a package from the server (http://35.187.91.110/vaaman/).
    pub async fn get_latest_version(
        name: &str,
    ) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
        let url = format!("http://35.187.91.110/vaaman/{name}/PKGBUILD");
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

    pub fn get_current_version(name: &str) -> std::io::Result<Option<(u8, u8, u8)>> {
        for package in read_vaaman_packages()?.packages {
            if package.name == name {
                return Ok(Some(package.version));
            }
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
}

impl VaamanPackages {
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
            let mut vaaman_packages = read_vaaman_packages()?;

            // check if package already exists
            if Package::get_current_version(&package.name)?.is_none() {
                vaaman_packages.add_package(package.clone());
            }

            let path = format!("/home/{}/.vaaman_packages.json", username());
            let json_string = serde_json::to_string_pretty(&vaaman_packages)?;
            let mut file = File::create(&path)?;
            file.write_all(json_string.as_bytes())?;
        }

        Ok(())
    }

    pub fn check_updates(&self) -> std::io::Result<Vec<Package>> {
        let mut updates = Vec::new();

        for package in &self.packages {
            println!("Checking for updates: {package}");
            if package.check_update()? {
                updates.push(package.clone());
            }
        }

        Ok(updates)
    }
}
