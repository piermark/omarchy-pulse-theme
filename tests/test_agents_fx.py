import importlib.machinery
import importlib.util
import io
import json
import subprocess
import tempfile
import time
import unittest
from unittest import mock
from pathlib import Path

path = Path(__file__).resolve().parents[1] / 'extras/pulse/omarchy-pulse'
loader = importlib.machinery.SourceFileLoader('pulse_agents', str(path))
spec = importlib.util.spec_from_loader(loader.name, loader)
pulse = importlib.util.module_from_spec(spec)
loader.exec_module(pulse)


class AgentTests(unittest.TestCase):
    def setUp(self):
        self.hub = pulse.Hub()

    def test_states_are_normalised(self):
        self.hub.set_agent('grok', 'thinking', 'reading')
        self.assertEqual(self.hub.agents['grok']['state'], 'working')
        self.hub.set_agent('grok', 'permission')
        self.assertEqual(self.hub.agents['grok']['state'], 'waiting')
        self.hub.set_agent('grok', 'nonsense')
        self.assertEqual(self.hub.agents['grok']['state'], 'working')

    def test_done_fires_fireworks_once(self):
        self.hub.set_agent('claude', 'working')
        self.hub.set_agent('claude', 'done')
        self.hub.set_agent('claude', 'done')
        self.assertEqual([f['kind'] for f in self.hub.fx], ['fireworks'])

    def test_failed_raises_an_alert(self):
        self.hub.set_agent('codex', 'failed', 'exit 2')
        self.assertEqual(self.hub.alerts[-1]['level'], 'warn')
        self.assertIn('codex', self.hub.alerts[-1]['title'])

    def test_gone_removes_and_names_are_cleaned(self):
        self.hub.set_agent('<b>evil</b>\n', 'working')
        name = next(iter(self.hub.agents))
        self.assertNotIn('<', name)
        self.hub.set_agent(name, 'gone')
        self.assertEqual(self.hub.agents, {})

    def test_finished_agents_expire(self):
        self.hub.set_agent('a', 'done')
        self.hub.agents['a']['t'] = time.time() - 120
        self.hub.expire_agents()
        self.assertNotIn('a', self.hub.agents)

    def test_focus_end_celebrates(self):
        self.hub.set_focus(1, 'write')
        self.hub.focus['until'] = time.time() - 1
        with mock.patch.object(pulse.shutil, 'which', return_value=None):
            self.hub.check_focus()
        self.assertIsNone(self.hub.focus)
        self.assertIn('write', self.hub.fx[-1]['text'])

    def test_back_to_green_is_a_small_burst(self):
        self.hub.set_widget('nas', title='nas', type='stat', data={'value': 1}, level='crit')
        self.hub.set_widget('nas', level='ok')
        self.assertEqual(self.hub.fx[-1]['kind'], 'burst')


class ColorTests(unittest.TestCase):
    def test_preset_and_override(self):
        hub = pulse.Hub()
        hub.cfg = {'colors': {'preset': 'matrix', 'accent': '#123456', 'warn': 'not-a-colour'}}
        c = hub.colors()['colors']
        self.assertEqual(c['accent'], '#123456')
        self.assertEqual(c['color3'], pulse.PRESETS['matrix']['warn'])

    def test_theme_by_default(self):
        hub = pulse.Hub()
        hub.theme = {'name': 'tokyo', 'colors': {'accent': '#abcdef'}}
        self.assertEqual(hub.colors()['colors']['accent'], '#abcdef')


class HookTests(unittest.TestCase):
    def run_hook(self, payload):
        sent = []
        with mock.patch.object(pulse, 'quiet_client', side_effect=lambda p, b=b'': sent.append(p) or 0), \
             mock.patch('sys.stdin', io.StringIO(json.dumps(payload))):
            pulse.agent_hook('claude', [])
        return sent

    def test_tool_use_is_working_without_the_prompt(self):
        sent = self.run_hook({'hook_event_name': 'PreToolUse', 'tool_name': 'Bash', 'cwd': '/home/u/omarchytv',
                              'tool_input': {'command': 'secret'}})
        self.assertEqual(len(sent), 1)
        self.assertIn('state=working', sent[0])
        self.assertNotIn('secret', sent[0])
        self.assertIn('omarchytv', sent[0])

    def test_prompt_text_never_leaves(self):
        sent = self.run_hook({'hook_event_name': 'UserPromptSubmit', 'prompt': 'my password is hunter2', 'cwd': '/x'})
        self.assertNotIn('hunter2', sent[0])

    def test_one_drone_per_repo_not_per_folder(self):
        # an agent that runs `cd apps/web` keeps its drone: the name and the group come from the repo, not the folder
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp) / 'myrepo'
            (repo / 'apps' / 'web').mkdir(parents=True)
            subprocess.run(['git', 'init', '-q', str(repo)], check=True)
            at_root = self.run_hook({'hook_event_name': 'PreToolUse', 'tool_name': 'Bash', 'cwd': str(repo)})[0]
            in_sub = self.run_hook({'hook_event_name': 'PreToolUse', 'tool_name': 'Bash', 'cwd': str(repo / 'apps' / 'web')})[0]
        self.assertEqual(at_root, in_sub)
        self.assertIn('claude%20%C2%B7%20myrepo?', in_sub)

    def test_stop_and_unknown(self):
        self.assertIn('state=done', self.run_hook({'hook_event_name': 'Stop', 'cwd': '/x'})[0])
        self.assertEqual(self.run_hook({'hook_event_name': 'PostToolUse', 'cwd': '/x'}), [])


if __name__ == '__main__':
    unittest.main()
