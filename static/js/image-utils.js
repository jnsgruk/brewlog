/** Convert any image file (HEIC, AVIF, WebP, PNG, etc.) to a JPEG data URL via Canvas.
 *  Uses createImageBitmap which correctly applies EXIF orientation (e.g. iPhone photos). */
const imageToJpegDataUrl = async (file) => {
  const bitmap = await createImageBitmap(file);
  const canvas = document.createElement("canvas");
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  canvas.getContext("2d").drawImage(bitmap, 0, 0);
  bitmap.close();
  return canvas.toDataURL("image/jpeg", 0.92);
};
