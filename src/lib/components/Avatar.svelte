<script lang="ts">
  let { id, size = 64 }: { id: string; size?: number } = $props();
  const images = import.meta.glob('../../../content/avatars/*.svg', {
    eager: true,
    query: '?url',
    import: 'default',
  }) as Record<string, string>;
  let source = $derived(
    images[
      `../../../content/avatars/${/^([0-9]|1[01])$/.test(id) ? id : '0'}.svg`
    ],
  );
</script>

<img src={source} alt="" width={size} height={size} />
