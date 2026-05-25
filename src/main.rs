use sysinfo::Components;
use std::thread;
use std::time::Duration;


fn main(){
    println!("Temperature sensors are being scanned...");
    let  components: Components=Components::new_with_refreshed_list();

    if components.list().is_empty(){
        println!("No readable temperature sensor was found on your computer.");
        println!("Not: Bazı işletim sistemlerinde (özellikle Windows) sıcaklık verilerine erişmek için uygulamayı Yönetici (Administrator) olarak çalıştırmanız gerekebilir.");
        return;
    }
    println!("Temperature readings are starting (Press Ctrl+C to exit)...\n");
    loop {
        for component in &components{
            if let Some(temp)=component.temperature(){
                println!("{}: {:.1}°C",component.label(),temp);
            }else {
                println!("{}: (Sıcaklık verisi okunamadı)", component.label());

            }
            println!("--------------------------\n");


            thread::sleep(Duration::from_secs(2));
        }

    }
}