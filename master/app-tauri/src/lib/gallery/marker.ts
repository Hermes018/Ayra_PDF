// The one place the gallery marker is defined. tools/check-no-gallery.sh reads it from here, so a rename
// cannot make the "absent from release" check pass vacuously (it also runs a positive control).
export const GALLERY_MARKER = 'LPDF_GALLERY_MARKER_7f3a';
