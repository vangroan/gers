

foreign class GraphicDevice {
  static instance { __instance }
  static instance_=(device) { __instance = device }

  // Create a new instance of graphic device, and register
  // it as the global singleton.
  construct new_() {
    System.print("GraphicDevice.new_()")

    if (__instance != null) {
      Fiber.abort("Only one graphics device allowed")
    }

    __instance = this
  }

  // foreign getViewport()
  foreign setViewport_(width, height)
  // foreign clear(rgba)
  foreign clearScreen(red, green, blue, alpha)
  foreign draw(batch)
  foreign draw(batch, transform)
  foreign maintain()
}
