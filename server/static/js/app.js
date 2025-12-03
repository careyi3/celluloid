async function initWasm() {
  try {
    const { default: init } = await import("/pkg/rust_wasm_animation.js");

    const canvas = document.getElementById("canvas");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    window.addEventListener("resize", () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    });

    await init();
    console.log("WASM module initialized successfully");
  } catch (error) {
    console.error("Failed to initialize WASM:", error);
    document.body.innerHTML = `
            <div style="color: white; padding: 20px; font-family: monospace;">
                <h1>Error Loading Animation</h1>
                <p>Failed to initialize WebAssembly module.</p>
                <pre>${error.message}</pre>
            </div>
        `;
  }
}

initWasm();
