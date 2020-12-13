import "window" for WindowConf

class Bootstrap {
  static window() {
    System.print("Window Configure")
    var config = WindowConf.new()
    config.set_size(1024, 768)
    config.set_title("Title from bootstrap")
    return config
  }

  static update(delta) {
    System.print(delta)
  }
}
