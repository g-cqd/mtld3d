# NFSMW macOS patches

This branch starts at athei/mtld3d 86a5bf42532624db8a9a0e2ab866ed4bf834d003. It contains the source changes used by the local NFSMW macOS preview on 2026-09-26. No game data is included.

## Immediate presentation pacing

Use an interruptible CPU deadline for an explicit frame cap when VSync is disabled. Avoid imposing the display refresh cadence through Metal minimum drawable duration in that mode. Preserve display-paced VSync. Unit tests cover cap cadence, late frames, uncapped operation and shutdown.

## Vertex spacing and read spans

Preserve a caller's nonzero vertex stride even if the consumed declaration extent is larger. Extend bounded staged-buffer read spans to include the final attribute, with overflow falling back to the whole tail. This supports overlapping layouts already accepted by this renderer; it does not claim that such layouts satisfy the documented Direct3D 9 stride contract.

The regression draws distinct green and red triangles at stride 12 and 44 and checks D3D9 read-back colors. Metal API validation accepted both pipelines on Apple M1. Unit tests cover zero, constant, per-instance, indexed and UP streams, plus overflow.

## Verification and limits

The final source passed make check and make test ISOLATED=1 on Apple M1, macOS 27 beta: 3957 core unit tests, 501 Unix tests, and 877 end-to-end tests per Wine architecture. Four benchmark tests per architecture were ignored. The production build passed 27 targeted stream/buffer tests. See CONTRIBUTING.md for the build environment and commands.

A 120 FPS limit is supported. Sustained 120 FPS racing has not been achieved. These patches do not implement DLSS, temporal reconstruction, frame generation or ray tracing. The existing renderer provides MetalFX spatial scaling.
