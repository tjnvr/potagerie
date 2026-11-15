<script lang="ts">
  // Logo of D01 (gen.py, LOGO).
  let { taille = 34 }: { taille?: number } = $props();
</script>

<svg width={taille} height={taille} viewBox="0 0 34 34" aria-hidden="true">
  <polygon points="17,4 6,12 7,22 17,30" fill="#6BBA62" />
  <polygon points="17,4 28,12 27,22 17,30" fill="#4A9450" />
  <polygon points="17,4 11,10 17,15 23,10" fill="#8ED17A" />
  <circle cx="21" cy="21" r="4" fill="#E5533D" />
  <circle cx="19.8" cy="19.8" r="1.2" fill="#fff" fill-opacity="0.6" />
</svg>
