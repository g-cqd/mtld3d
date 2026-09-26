# NFSMW macOS patches

This branch rebases the four NFSMW patch and documentation commits onto athei/mtld3d 1b0bf1f40c8bfeecb4b0c5a2c8182ef6893edd76. It contains the source changes used by the local NFSMW macOS preview on 2026-09-26. No game data is included.

## Immediate presentation pacing

Use an interruptible CPU deadline for an explicit frame cap when VSync is disabled. Avoid imposing the display refresh cadence through Metal minimum drawable duration in that mode. Preserve display-paced VSync. Unit tests cover cap cadence, late frames, uncapped operation and shutdown.

## Vertex spacing and read spans

Preserve a caller's nonzero vertex stride even if the consumed declaration extent is larger. Extend bounded staged-buffer read spans to include the final attribute, with overflow falling back to the whole tail. This supports overlapping layouts already accepted by this renderer; it does not claim that such layouts satisfy the documented Direct3D 9 stride contract.

The regression draws distinct green and red triangles at stride 12 and 44 and checks D3D9 read-back colors. Metal API validation accepted both pipelines on Apple M1. Unit tests cover zero, constant, per-instance, indexed and UP streams, plus overflow.

## Verification and limits

The final source passed make check and make test ISOLATED=1 on Apple M1, macOS 27 beta: 1557 Windows host tests, 642 Unix host tests, and 881 end-to-end tests per Wine architecture. Each architecture reported 892 tests total, 11 ignored, zero failures and zero unaccounted tests. The production build and isolated installation passed with PROD=1 FP=1. See CONTRIBUTING.md for the build environment and commands.

A 120 FPS limit is supported. Sustained 120 FPS racing has not been achieved. These patches do not implement DLSS, temporal reconstruction, frame generation or ray tracing. The existing renderer provides MetalFX spatial scaling.

## Upstream refresh

The `nfsmw-upstream-update` candidate contains the retained-stencil correction, test-window shutdown correction and regression benchmark harness from upstream. The original `nfsmw-macos` branch remains available. This update has not been shown to increase game FPS.
