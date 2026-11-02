# Interrupted input capture

The calling turn interrupted this capture before a complete input receipt or
inspection result existed. No inspection worker started. The exact incomplete
`inputs.tar.gz` remains physically retained and is identified in the
[interruption record](../interruption.json).

The [storage manifest](../archive-storage/manifest.json) also retains every byte
and mode of this incomplete container in reversible chunks. It remains an
incomplete construction record, with no complete-archive or successful-check
claim. This tracked description retains the restoration directory; the storage
driver can restore its exact local compressed copy from the chunks.
