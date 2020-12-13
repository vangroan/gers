
foreign class Window {
    construct new(width, height) {}
    foreign run()
}

foreign class WindowConf {
  construct new() {}
  foreign set_size(width, height)
  foreign set_title(title)
}
