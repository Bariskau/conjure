<script setup lang="ts">
import { computed } from "vue";

export type IconName =
  | "arrow-right"
  | "arrowright"
  | "bolt"
  | "check"
  | "chevron"
  | "chevron-down"
  | "chevron-right"
  | "chevdown"
  | "clock"
  | "close"
  | "code"
  | "copy"
  | "edit"
  | "folder"
  | "gear"
  | "logs"
  | "pencil"
  | "play"
  | "plus"
  | "refresh"
  | "save"
  | "search"
  | "settings"
  | "spinner"
  | "trash"
  | "x"
  | "zap";

const props = withDefaults(
  defineProps<{
    name: IconName;
    size?: number;
    strokeWidth?: number;
    fill?: boolean;
  }>(),
  {
    size: 18,
    strokeWidth: 1.6,
    fill: false,
  },
);

const path = computed(() => ICON_PATHS[aliasForName(props.name)] ?? "");

const ICON_PATHS = {
  search: "M11 11m-7 0a7 7 0 1014 0a7 7 0 10-14 0M20 20l-4-4",
  plus: "M12 5v14M5 12h14",
  gear: "M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2zM12 15a3 3 0 100-6 3 3 0 000 6",
  play: "M7 5l12 7-12 7z",
  edit: "M4 20h4l10-10-4-4L4 16zM14 6l4 4",
  check: "M5 12.5l4.5 4.5L19 7",
  x: "M6 6l12 12M18 6L6 18",
  clock: "M12 12m-9 0a9 9 0 1018 0a9 9 0 10-18 0M12 7v5l3 2",
  chevron: "M9 6l6 6-6 6",
  chevdown: "M6 9l6 6 6-6",
  trash: "M5 7h14M9 7V5h6v2M7 7l1 13h8l1-13",
  copy: "M9 9h10v10H9zM5 15V5h10",
  arrowright: "M5 12h14M13 6l6 6-6 6",
  spinner: "M12 3a9 9 0 109 9",
  code: "M9 8l-4 4 4 4M15 8l4 4-4 4",
  bolt: "M13 3L4 14h6l-1 7 9-11h-6z",
  close: "M6 6l12 12M18 6L6 18",
  refresh: "M20 11a8 8 0 10-2 5M20 5v6h-6",
  folder: "M3.5 7.5a2 2 0 012-2h3.6l2 2H18.5a2 2 0 012 2v7a2 2 0 01-2 2h-13a2 2 0 01-2-2z",
  logs: "M5 4h11l3 3v13H5zM9 11h7M9 15h7M9 7h4",
} as const;

function aliasForName(name: IconName): keyof typeof ICON_PATHS {
  const aliases: Record<IconName, keyof typeof ICON_PATHS> = {
    "arrow-right": "arrowright",
    arrowright: "arrowright",
    bolt: "bolt",
    check: "check",
    chevron: "chevron",
    "chevron-down": "chevdown",
    "chevron-right": "chevron",
    chevdown: "chevdown",
    clock: "clock",
    close: "close",
    code: "code",
    copy: "copy",
    edit: "edit",
    folder: "folder",
    gear: "gear",
    logs: "logs",
    pencil: "edit",
    play: "play",
    plus: "plus",
    refresh: "refresh",
    save: "check",
    search: "search",
    settings: "gear",
    spinner: "spinner",
    trash: "trash",
    x: "x",
    zap: "bolt",
  };

  return aliases[name];
}
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    aria-hidden="true"
    class="app-icon"
    :fill="fill ? 'currentColor' : 'none'"
    :stroke="fill ? 'none' : 'currentColor'"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path :d="path" />
  </svg>
</template>

<style scoped>
.app-icon {
  display: block;
  flex-shrink: 0;
}
</style>
