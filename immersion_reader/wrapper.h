// FBInk wrapper header for bindgen
// This is a minimal declaration - on the actual device you'd use the real FBInk headers

#ifndef FBINK_WRAPPER_H
#define FBINK_WRAPPER_H

#include <stdint.h>
#include <stdbool.h>

// Forward declarations for FBInk types
typedef struct FBInkConfig FBInkConfig;
typedef struct FBInkState FBInkState;
typedef struct FBInkRect FBInkRect;
typedef struct FBInkOTConfig FBInkOTConfig;

// Waveform modes
#define FBINK_WAVEFORM_AUTO 0
#define FBINK_WAVEFORM_DU 1
#define FBINK_WAVEFORM_GC16 2
#define FBINK_WAVEFORM_GL16 3
#define FBINK_WAVEFORM_A2 4
#define FBINK_WAVEFORM_GC4 5

// Core FBInk structures (simplified)
struct FBInkConfig {
    short int row;
    short int col;
    uint8_t fontmult;
    uint8_t fontname;
    bool is_inverted;
    bool is_flashing;
    bool is_cleared;
    bool is_centered;
    int8_t hoffset;
    int8_t voffset;
    bool is_halfway;
    bool is_padded;
    bool is_rpadded;
    bool fg_color;
    bool bg_color;
    bool is_overlay;
    bool is_bgless;
    bool is_fgless;
    bool no_viewport;
    bool ignore_alpha;
    uint8_t halign;
    uint8_t valign;
    short int scaled_width;
    short int scaled_height;
    int8_t wfm_mode;
    int8_t dithering_mode;
    bool is_nightmode;
    bool no_refresh;
};

struct FBInkState {
    uint32_t view_width;
    uint32_t view_height;
    uint32_t screen_width;
    uint32_t screen_height;
    uint8_t bpp;
    char device_name[32];
    uint8_t device_id;
    uint8_t pen_fg_color;
    uint8_t pen_bg_color;
    uint16_t screen_dpi;
    uint16_t font_w;
    uint16_t font_h;
    uint16_t max_cols;
    uint16_t max_rows;
    uint8_t view_hori_origin;
    uint8_t view_vert_origin;
    uint8_t view_vert_offset;
    uint8_t fontsize_mult;
    uint8_t glyph_width;
    uint8_t glyph_height;
    bool is_perfect_fit;
    uint8_t ntx_boot_rota;
    uint8_t ntx_rota_quirk;
    bool is_ntx_quirky;
    uint8_t current_rota;
    uint8_t can_rotate;
    uint32_t scanline_stride;
};

struct FBInkRect {
    unsigned short int left;
    unsigned short int top;
    unsigned short int width;
    unsigned short int height;
};

struct FBInkOTConfig {
    float size_pt;
    unsigned short int size_px;
    bool is_centered;
    short int padding;
    bool is_formatted;
    bool compute_only;
    bool no_truncation;
    FBInkRect margins;
};

// Function declarations
int fbink_init(int fbfd, const FBInkConfig* restrict fbink_cfg);
int fbink_print(int fbfd, const char* restrict string, const FBInkConfig* restrict fbink_cfg);
int fbink_refresh(int fbfd, uint32_t region_top, uint32_t region_left,
                  uint32_t region_width, uint32_t region_height,
                  const FBInkConfig* restrict fbink_cfg);
void fbink_get_state(const FBInkConfig* restrict fbink_cfg, FBInkState* restrict fbink_state);
int fbink_add_ot_font(const char* restrict filename, int fbfd);
const char* fbink_version(void);
int fbink_close(int fbfd);
int fbink_get_last_rect(FBInkRect* restrict rect);
int fbink_cls(int fbfd, const FBInkConfig* restrict fbink_cfg, const FBInkRect* restrict region);

#endif // FBINK_WRAPPER_H
