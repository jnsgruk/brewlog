/** Convert any image file (HEIC, AVIF, WebP, PNG, etc.) to a JPEG data URL via Canvas.
 *  Uses createImageBitmap which correctly applies EXIF orientation (e.g. iPhone photos).
 *  Caps the longest side to 1920px to avoid exceeding the request body limit. */
const imageToJpegDataUrl = async (file) => {
  const bitmap = await createImageBitmap(file);
  const maxDim = 1920;
  let { width, height } = bitmap;
  if (width > maxDim || height > maxDim) {
    const scale = maxDim / Math.max(width, height);
    width = Math.round(width * scale);
    height = Math.round(height * scale);
  }
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  canvas.getContext("2d").drawImage(bitmap, 0, 0, width, height);
  bitmap.close();
  return canvas.toDataURL("image/jpeg", 0.92);
};
