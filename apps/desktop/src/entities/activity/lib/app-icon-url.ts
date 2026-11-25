/**
 * The desktop host serves app icons on its `appicon` URI scheme. WebView2 on
 * Windows reaches custom schemes as `http://<scheme>.localhost`.
 */
export function appIconUrl(appId: string) {
  const path = encodeURIComponent(appId)
  return navigator.userAgent.includes("Windows")
    ? `http://appicon.localhost/${path}`
    : `appicon://localhost/${path}`
}
