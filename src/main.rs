pub mod update;
use update::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let available_packages = vec!["vicharak-firmware", "vicharak-config"];
    let mut packages = VicharakPackages::new();

    for package in available_packages {
        match Package::get_host_os_type() {
            OSType::Arch => {
                packages.add_package(Package::new(
                    package.to_string(),
                    Package::get_latest_version_arch(package).await?,
                ));
            }
            OSType::Debian => {
                packages.add_package(Package::new(
                    package.to_string(),
                    Package::get_latest_version_debian(package).await?,
                ));
            }
            OSType::Unknown => {
                println!("Unknown OS");
            }
        }
    }

    let update = packages.check_updates()?;

    if update.is_empty() {
        println!("No updates available");
    } else {
        println!("Updates available");

        for package in update {
            println!("Updating {}", package.name);
            package.update_package()?;
        }
    }

    packages.save_packages_to_file()?;

    Ok(())
}
