pub mod update;
use update::Package;
use update::VaamanPackages;

#[tokio::main]
async fn main() {
    let available_packages = vec!["vaaman-aes", "vaamos-menu"];
    let mut packages = VaamanPackages::new();

    for package in available_packages {
        packages.add_package(Package::new(
            package.to_string(),
            Package::get_latest_version(package).await.unwrap(),
        ));
    }
    packages.check_updates().unwrap();
    packages.save_packages_to_file().unwrap();
}
