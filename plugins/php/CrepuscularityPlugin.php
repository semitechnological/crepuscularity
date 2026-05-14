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

    public static function renderHtml(string $path): string
    {
        return implode('', array_map([self::class, 'renderNode'], self::renderIr($path)['root']));
    }

    private static function renderNode(array $node): string
    {
        return match ($node['kind'] ?? '') {
            'text' => htmlspecialchars((string) ($node['content'] ?? ''), ENT_QUOTES),
            'stack', 'scroll' => '<div data-crepus-kind="' . htmlspecialchars((string) $node['kind'], ENT_QUOTES) . '" data-axis="' . htmlspecialchars((string) ($node['axis'] ?? 'column'), ENT_QUOTES) . '">' . implode('', array_map([self::class, 'renderNode'], $node['children'] ?? [])) . '</div>',
            'button' => '<button>' . htmlspecialchars((string) ($node['label'] ?? ''), ENT_QUOTES) . '</button>',
            default => '',
        };
    }
}
