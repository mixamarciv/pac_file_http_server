function FindProxyForURL(url, host) {
    if (dnsDomainIs(host, "www.guimp.com") || dnsDomainIs(host, "poisk123.ru")) {
        return "PROXY 127.0.0.1:7001";
    }
    return "DIRECT";
}
