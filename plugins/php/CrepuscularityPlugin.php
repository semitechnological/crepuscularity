<?php

final class CrepuscularityPlugin
{
    public static function renderIr(string $path): array
    {
        $bin = getenv('CREPUS_BIN') ?: 'crepus';
        $cmd = escapeshellcmd($bin) . ' native ir ' . escapeshellarg($path);
        $out = [];
        $code = 0;
        exec($cmd, $out, $code);
        if ($code !== 0) {
            throw new RuntimeException('crepus native ir failed');
        }
        return json_decode(implode("\n", $out), true, 512, JSON_THROW_ON_ERROR);
    }
}
