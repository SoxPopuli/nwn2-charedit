#include "bc7decomp.h"

extern "C" bool unpack_bc7(const void* pBlock, bc7decomp::color_rgba* pPixels) {
    return bc7decomp::unpack_bc7(pBlock, pPixels);
}
