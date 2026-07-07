<?php

require __DIR__ . '/CrepuscularityPlugin.php';

$fixture = dirname(__DIR__) . '/fixtures/interactive.crepus';
$session = new CrepusViewSession($fixture, ['count' => '1']);

if (!str_contains($session->renderHtml(), 'Count 1')) {
    throw new RuntimeException('initial render did not include Count 1');
}

$ir = $session->dispatch('bind:count:2');

if (($session->context['count'] ?? null) !== '2') {
    throw new RuntimeException('dispatch did not update context');
}

if (!str_contains(json_encode($ir, JSON_THROW_ON_ERROR), 'Count 2')) {
    throw new RuntimeException('rerender did not include Count 2');
}

if (!str_contains($session->renderHtml(), 'Count 2')) {
    throw new RuntimeException('html rerender did not include Count 2');
}

$reflection = new ReflectionClass(CrepuscularityPlugin::class);
$method = $reflection->getMethod('crepusBin');
$method->setAccessible(true);

putenv('CREPUS_BIN=crepus');
if ($method->invoke(null) !== 'crepus') {
    throw new RuntimeException('failed to allow simple binary name');
}

putenv('CREPUS_BIN=/usr/local/bin/crepus');
if ($method->invoke(null) !== '/usr/local/bin/crepus') {
    throw new RuntimeException('failed to allow absolute path');
}

putenv('CREPUS_BIN=../../evil');
$threw = false;
try {
    $method->invoke(null);
} catch (RuntimeException $e) {
    $threw = true;
}
if (!$threw) {
    throw new RuntimeException('failed to reject relative path traversal');
}
