use hyprbar::bar::system::{
    LinkStatus, band_label, channel_for_freq, parse_active_security, parse_default_route, parse_df_bytes, parse_interface_bytes,
    parse_ipv4_with_prefix, parse_iw_link, parse_load_average, parse_mirror_host, parse_nameservers, parse_pending_packages, parse_ping_ms,
    parse_swap_used_kb,
};

#[test]
fn parses_default_route_and_little_endian_gateway() {
    let route = "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\n\
                 wlan0\t00000000\t0101A8C0\t0003\t0\t0\t600\t00000000\t0\t0\t0\n\
                 wlan0\t0001A8C0\t00000000\t0001\t0\t0\t600\t00FFFFFF\t0\t0\t0\n";

    let (interface, gateway) = parse_default_route(route).expect("ruta por defecto");

    assert_eq!(interface, "wlan0");
    assert_eq!(gateway, "192.168.1.1");
}

#[test]
fn returns_none_without_default_route() {
    let route = "Iface\tDestination\tGateway\n\
                 eth0\t0001A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0\n";

    assert_eq!(parse_default_route(route), None);
}

#[test]
fn parses_interface_counters_from_net_dev() {
    let net_dev = "Inter-|   Receive                                                |  Transmit\n\
                   face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n\
                   lo: 1000 10 0 0 0 0 0 0 2000 20 0 0 0 0 0 0\n\
                   wlan0: 4400000000 3000000 0 0 0 0 0 0 640000000 900000 0 0 0 0 0 0\n";

    assert_eq!(parse_interface_bytes(net_dev, "wlan0"), Some((4_400_000_000, 640_000_000)));
    assert_eq!(parse_interface_bytes(net_dev, "eth0"), None);
}

#[test]
fn parses_iw_link_output() {
    let output = "Connected to aa:bb:cc:dd:ee:ff (on wlan0)\n\
                  \tSSID: casa-5G\n\
                  \tfreq: 5220.0\n\
                  \tsignal: -48 dBm\n\
                  \trx bitrate: 866.7 MBit/s\n";

    let status = parse_iw_link(output);

    assert_eq!(
        status,
        LinkStatus {
            ssid: Some("casa-5G".to_string()),
            signal_dbm: Some(-48),
            freq_mhz: Some(5220),
        }
    );
}

#[test]
fn maps_frequency_to_band_and_channel() {
    assert_eq!(band_label(2437), "2.4 GHz");
    assert_eq!(band_label(5220), "5 GHz");
    assert_eq!(band_label(5975), "6 GHz");

    assert_eq!(channel_for_freq(2437), Some(6));
    assert_eq!(channel_for_freq(5220), Some(44));
    assert_eq!(channel_for_freq(2484), Some(14));
    assert_eq!(channel_for_freq(100), None);
}

#[test]
fn parses_active_wifi_security() {
    let output = "no:WPA2\nyes:WPA2 WPA3\nno:--\n";

    assert_eq!(parse_active_security(output), Some("WPA3".to_string()));
    assert_eq!(parse_active_security("no:WPA2\n"), None);
}

#[test]
fn parses_ipv4_with_prefix_from_ip_output() {
    let output = "2: wlan0    inet 192.168.1.24/24 brd 192.168.1.255 scope global dynamic noprefixroute wlan0";

    assert_eq!(parse_ipv4_with_prefix(output), Some("192.168.1.24/24".to_string()));
    assert_eq!(parse_ipv4_with_prefix("sin nada"), None);
}

#[test]
fn parses_nameservers() {
    let resolv = "# generado\nnameserver 1.1.1.1\nnameserver 9.9.9.9\nsearch lan\n";

    assert_eq!(parse_nameservers(resolv), vec!["1.1.1.1".to_string(), "9.9.9.9".to_string()]);
}

#[test]
fn parses_ping_round_trip() {
    let output = "64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=34.2 ms\n";

    assert_eq!(parse_ping_ms(output), Some(34.2));
    assert_eq!(parse_ping_ms("timeout"), None);
}

#[test]
fn parses_load_average() {
    assert_eq!(parse_load_average("0.18 0.24 0.30 1/500 12345\n").unwrap(), 0.18);
    assert!(parse_load_average("").is_err());
}

#[test]
fn parses_swap_used() {
    let meminfo = "SwapTotal:       8000000 kB\nSwapFree:        7500000 kB\n";

    assert_eq!(parse_swap_used_kb(meminfo).unwrap(), 500_000);
}

#[test]
fn parses_df_output() {
    let output = "    Used   1B-blocks\n214000000000 476000000000\n";

    assert_eq!(parse_df_bytes(output).unwrap(), (214_000_000_000, 476_000_000_000));
    assert!(parse_df_bytes("solo header\n").is_err());
}

#[test]
fn parses_pending_packages_from_checkupdates() {
    let stdout = "linux 6.16.1-arch1 -> 6.16.2-arch1\nmesa 25.2.0-1 -> 25.2.1-1\n";

    let packages = parse_pending_packages(stdout);

    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].name, "linux");
    assert_eq!(packages[0].version, "6.16.2-arch1");
    assert_eq!(packages[1].name, "mesa");
    assert_eq!(packages[1].version, "25.2.1-1");
}

#[test]
fn parses_mirror_host() {
    let mirrorlist = "# comentario\n#Server = https://viejo.example/$repo\nServer = https://mirror.archlinux.org/$repo/os/$arch\n";

    assert_eq!(parse_mirror_host(mirrorlist), Some("mirror.archlinux.org".to_string()));
}
