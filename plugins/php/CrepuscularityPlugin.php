<?php

final class CrepuscularityPlugin
{
    public static function renderIr(string $path, ?array $context = null): array
    {
        $bin = getenv('CREPUS_BIN') ?: 'crepus';
        if ($context !== null) {
            $payload = json_encode([
                'template' => file_get_contents($path),
                'context' => $context,
                'baseDir' => dirname($path),
            ], JSON_THROW_ON_ERROR);
            $descriptor = [
                0 => ['pipe', 'r'],
                1 => ['pipe', 'w'],
                2 => ['pipe', 'w'],
            ];
            $proc = proc_open([$bin, 'native', 'ir', '--stdin-json'], $descriptor, $pipes);
            if (!is_resource($proc)) {
                throw new RuntimeException('crepus native ir failed');
            }
            fwrite($pipes[0], $payload);
            fclose($pipes[0]);
            $stdout = stream_get_contents($pipes[1]);
            fclose($pipes[1]);
            $stderr = stream_get_contents($pipes[2]);
            fclose($pipes[2]);
            $code = proc_close($proc);
            if ($code !== 0) {
                throw new RuntimeException(trim($stderr));
            }
            return json_decode($stdout, true, 512, JSON_THROW_ON_ERROR);
        }
        $cmd = escapeshellcmd($bin) . ' native ir ' . escapeshellarg($path);
        $out = [];
        $code = 0;
        exec($cmd, $out, $code);
        if ($code !== 0) {
            throw new RuntimeException('crepus native ir failed');
        }
        return json_decode(implode("\n", $out), true, 512, JSON_THROW_ON_ERROR);
    }

    public static function renderHtml(string $path, ?array $context = null): string
    {
        return implode('', array_map([self::class, 'renderNode'], self::renderIr($path, $context)['root']));
    }

    private static function renderNode(array $node): string
    {
        return match ($node['kind'] ?? '') {
            'text' => htmlspecialchars((string) ($node['content'] ?? ''), ENT_QUOTES),
            'stack', 'scroll' => '<div data-crepus-kind="' . htmlspecialchars((string) $node['kind'], ENT_QUOTES) . '" data-axis="' . htmlspecialchars((string) ($node['axis'] ?? 'column'), ENT_QUOTES) . '">' . implode('', array_map([self::class, 'renderNode'], $node['children'] ?? [])) . '</div>',
            'button' => '<button' . (isset($node['onClick']) ? ' data-onclick="' . htmlspecialchars((string) $node['onClick'], ENT_QUOTES) . '"' : '') . '>' . htmlspecialchars((string) ($node['label'] ?? ''), ENT_QUOTES) . '</button>',
            'image' => '<img src="' . htmlspecialchars((string) ($node['src'] ?? ''), ENT_QUOTES) . '" alt="' . htmlspecialchars((string) ($node['alt'] ?? ''), ENT_QUOTES) . '">',
            'slotRotate' => '<span data-crepus-kind="slotRotate">' . htmlspecialchars((string) (($node['phrases'] ?? [''])[0] ?? ''), ENT_QUOTES) . '</span>',
            'input' => !empty($node['multiline'])
                ? '<textarea data-bind="' . htmlspecialchars((string) ($node['bind'] ?? ''), ENT_QUOTES) . '" placeholder="' . htmlspecialchars((string) ($node['placeholder'] ?? ''), ENT_QUOTES) . '"></textarea>'
                : '<input data-bind="' . htmlspecialchars((string) ($node['bind'] ?? ''), ENT_QUOTES) . '" placeholder="' . htmlspecialchars((string) ($node['placeholder'] ?? ''), ENT_QUOTES) . '">',
            'picker' => '<select data-bind="' . htmlspecialchars((string) ($node['bind'] ?? ''), ENT_QUOTES) . '">' . implode('', array_map(fn (array $option): string => '<option value="' . htmlspecialchars((string) ($option['value'] ?? ''), ENT_QUOTES) . '">' . htmlspecialchars((string) ($option['label'] ?? ''), ENT_QUOTES) . '</option>', $node['options'] ?? [])) . '</select>',
            default => '',
        };
    }
}

final class CrepusViewSession
{
    public string $path;
    public array $context;
    private array $handlers = [];

    public function __construct(string $path, array $context = [])
    {
        $this->path = $path;
        $this->context = $context;
    }

    public function on(string $handler, callable $callback): self
    {
        $this->handlers[$handler] = $callback;
        return $this;
    }

    public function renderIr(): array
    {
        return CrepuscularityPlugin::renderIr($this->path, $this->context);
    }

    public function renderHtml(): string
    {
        return CrepuscularityPlugin::renderHtml($this->path, $this->context);
    }

    public function dispatch(string|array $event): array
    {
        $payload = is_string($event) ? ['handler' => $event] : $event;
        $handler = (string) ($payload['handler'] ?? '');
        if (str_starts_with($handler, 'bind:')) {
            $parts = explode(':', substr($handler, 5), 2);
            if (count($parts) === 2) {
                $this->context[$parts[0]] = $parts[1];
            }
        }
        if (isset($this->handlers[$handler])) {
            ($this->handlers[$handler])($payload, $this);
        }
        return $this->renderIr();
    }
}
