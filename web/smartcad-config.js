// SmartCAD: marca y pantalla de inicio. Se lee al arrancar (src/smartcad.rs).
// Editar y publicar este archivo NO requiere recompilar el wasm:
//   SKIP_BUILD=1 sh deploy/cloudflare/publish.sh
window.SMARTCAD_CONFIG = {
  appName: "SmartCAD",
  start: {
    showTitle: false,
    showSupporters: false,
    showDiscussions: false,
    showVideos: true,
    showDonate: false,
    showReddit: false,
    showSponsors: true,
    showDesktopLink: false,
  },
  donationPrompt: false,
};
