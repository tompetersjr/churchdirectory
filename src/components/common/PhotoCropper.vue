<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

const props = defineProps<{
  imageSrc: string;
  loading?: boolean;
}>();

const emit = defineEmits<{
  crop: [cropData: { x: number; y: number; width: number; height: number }];
  cancel: [];
}>();

const containerRef = ref<HTMLDivElement | null>(null);

const imageNaturalWidth = ref(0);
const imageNaturalHeight = ref(0);

// Crop box state (in percentage 0-100)
const cropBox = ref({
  x: 25,
  y: 25,
  width: 50,
  height: 50,
});

const isDragging = ref(false);
const isResizing = ref(false);
const resizeHandle = ref<string | null>(null);
const dragStart = ref({ x: 0, y: 0, boxX: 0, boxY: 0, boxW: 0, boxH: 0 });

const cropBoxStyle = computed(() => ({
  left: `${cropBox.value.x}%`,
  top: `${cropBox.value.y}%`,
  width: `${cropBox.value.width}%`,
  height: `${cropBox.value.height}%`,
}));

function onImageLoad(event: Event) {
  const img = event.target as HTMLImageElement;
  imageNaturalWidth.value = img.naturalWidth;
  imageNaturalHeight.value = img.naturalHeight;
  console.log("Image loaded:", img.naturalWidth, "x", img.naturalHeight);
}

function startDrag(event: MouseEvent) {
  if (isResizing.value) return;
  isDragging.value = true;
  dragStart.value = {
    x: event.clientX,
    y: event.clientY,
    boxX: cropBox.value.x,
    boxY: cropBox.value.y,
    boxW: cropBox.value.width,
    boxH: cropBox.value.height,
  };
  event.preventDefault();
}

function startResize(handle: string, event: MouseEvent) {
  isResizing.value = true;
  resizeHandle.value = handle;
  dragStart.value = {
    x: event.clientX,
    y: event.clientY,
    boxX: cropBox.value.x,
    boxY: cropBox.value.y,
    boxW: cropBox.value.width,
    boxH: cropBox.value.height,
  };
  event.preventDefault();
  event.stopPropagation();
}

function onMouseMove(event: MouseEvent) {
  if (!containerRef.value) return;

  const rect = containerRef.value.getBoundingClientRect();
  const deltaXPercent = ((event.clientX - dragStart.value.x) / rect.width) * 100;
  const deltaYPercent = ((event.clientY - dragStart.value.y) / rect.height) * 100;

  if (isDragging.value) {
    let newX = dragStart.value.boxX + deltaXPercent;
    let newY = dragStart.value.boxY + deltaYPercent;

    newX = Math.max(0, Math.min(100 - cropBox.value.width, newX));
    newY = Math.max(0, Math.min(100 - cropBox.value.height, newY));

    cropBox.value.x = newX;
    cropBox.value.y = newY;
  } else if (isResizing.value && resizeHandle.value) {
    const handle = resizeHandle.value;
    let newX = dragStart.value.boxX;
    let newY = dragStart.value.boxY;
    let newW = dragStart.value.boxW;
    let newH = dragStart.value.boxH;

    if (handle.includes("e")) {
      newW = Math.max(10, Math.min(100 - dragStart.value.boxX, dragStart.value.boxW + deltaXPercent));
    }
    if (handle.includes("w")) {
      const maxDelta = dragStart.value.boxX;
      const clampedDelta = Math.max(-maxDelta, Math.min(dragStart.value.boxW - 10, deltaXPercent));
      newX = dragStart.value.boxX + clampedDelta;
      newW = dragStart.value.boxW - clampedDelta;
    }
    if (handle.includes("s")) {
      newH = Math.max(10, Math.min(100 - dragStart.value.boxY, dragStart.value.boxH + deltaYPercent));
    }
    if (handle.includes("n")) {
      const maxDelta = dragStart.value.boxY;
      const clampedDelta = Math.max(-maxDelta, Math.min(dragStart.value.boxH - 10, deltaYPercent));
      newY = dragStart.value.boxY + clampedDelta;
      newH = dragStart.value.boxH - clampedDelta;
    }

    cropBox.value = { x: newX, y: newY, width: newW, height: newH };
  }
}

function onMouseUp() {
  isDragging.value = false;
  isResizing.value = false;
  resizeHandle.value = null;
}

function confirmCrop() {
  // Convert percentage-based crop box to actual pixel coordinates
  const x = Math.round((cropBox.value.x / 100) * imageNaturalWidth.value);
  const y = Math.round((cropBox.value.y / 100) * imageNaturalHeight.value);
  const width = Math.round((cropBox.value.width / 100) * imageNaturalWidth.value);
  const height = Math.round((cropBox.value.height / 100) * imageNaturalHeight.value);

  console.log("Crop data:", { x, y, width, height });
  console.log("From box:", cropBox.value);
  console.log("Image size:", imageNaturalWidth.value, "x", imageNaturalHeight.value);

  emit("crop", { x, y, width, height });
}

onMounted(() => {
  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
});

onUnmounted(() => {
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
});
</script>

<template>
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg shadow-xl max-w-3xl w-full mx-4 overflow-hidden">
      <div class="p-4 border-b flex items-center justify-between">
        <h3 class="text-lg font-semibold">Crop Member Photo</h3>
        <button @click="emit('cancel')" class="text-gray-400 hover:text-gray-600">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="p-4">
        <!-- Image container with crop overlay -->
        <div
          ref="containerRef"
          class="relative mx-auto select-none"
          style="max-width: 500px;"
        >
          <img
            :src="imageSrc"
            class="w-full h-auto"
            @load="onImageLoad"
            draggable="false"
          />

          <!-- Darkened overlay outside crop area -->
          <div class="absolute inset-0 pointer-events-none">
            <div class="absolute inset-0 bg-black/50" />
            <div
              class="absolute bg-transparent"
              :style="{
                left: `${cropBox.x}%`,
                top: `${cropBox.y}%`,
                width: `${cropBox.width}%`,
                height: `${cropBox.height}%`,
                boxShadow: '0 0 0 9999px rgba(0,0,0,0.5)',
              }"
            />
          </div>

          <!-- Crop box -->
          <div
            class="absolute border-2 border-white cursor-move"
            :style="cropBoxStyle"
            @mousedown="startDrag"
          >
            <!-- Corner resize handles -->
            <div
              class="absolute -top-1.5 -left-1.5 w-3 h-3 bg-white border border-gray-400 cursor-nw-resize"
              @mousedown="startResize('nw', $event)"
            />
            <div
              class="absolute -top-1.5 -right-1.5 w-3 h-3 bg-white border border-gray-400 cursor-ne-resize"
              @mousedown="startResize('ne', $event)"
            />
            <div
              class="absolute -bottom-1.5 -left-1.5 w-3 h-3 bg-white border border-gray-400 cursor-sw-resize"
              @mousedown="startResize('sw', $event)"
            />
            <div
              class="absolute -bottom-1.5 -right-1.5 w-3 h-3 bg-white border border-gray-400 cursor-se-resize"
              @mousedown="startResize('se', $event)"
            />
            <!-- Edge resize handles -->
            <div
              class="absolute -top-1.5 left-1/2 -translate-x-1/2 w-6 h-3 bg-white border border-gray-400 cursor-n-resize"
              @mousedown="startResize('n', $event)"
            />
            <div
              class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-6 h-3 bg-white border border-gray-400 cursor-s-resize"
              @mousedown="startResize('s', $event)"
            />
            <div
              class="absolute top-1/2 -left-1.5 -translate-y-1/2 w-3 h-6 bg-white border border-gray-400 cursor-w-resize"
              @mousedown="startResize('w', $event)"
            />
            <div
              class="absolute top-1/2 -right-1.5 -translate-y-1/2 w-3 h-6 bg-white border border-gray-400 cursor-e-resize"
              @mousedown="startResize('e', $event)"
            />
          </div>
        </div>

        <p class="text-sm text-gray-500 text-center mt-3">
          Drag to move the crop area. Use the handles to resize.
        </p>
      </div>

      <div class="p-4 border-t flex justify-end gap-3">
        <button
          @click="emit('cancel')"
          :disabled="props.loading"
          class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          @click="confirmCrop"
          :disabled="props.loading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 flex items-center gap-2"
        >
          <span v-if="props.loading" class="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></span>
          {{ props.loading ? 'Applying...' : 'Apply to Member' }}
        </button>
      </div>
    </div>
  </div>
</template>
