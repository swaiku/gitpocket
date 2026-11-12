<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import * as Card from '$lib/components/ui/card';
  import * as Tabs from '$lib/components/ui/tabs';
  import * as Select from '$lib/components/ui/select';
  import { filterRepos, trackAdditions, type Overview, type Seen, type View } from '$lib/filters';

  const views: [View, string][] = [['collaborators', 'Collaborations'], ['shared', 'Shared'], ['all', 'All'], ['invitations', 'Invitations']];
  const sorts = { pushed: 'Last push', added: 'Recently detected', updated: 'Last update', name: 'Repository name' };
  const visibilities = { '': 'Public and private', private: 'Private', public: 'Public' };

  let data = $state<Overview | null>(null);
  let seen = $state<Seen>({});
  let view = $state<View>('all');
  let query = $state('');
  let owner = $state('');
  let visibility = $state('');
  let sort = $state('pushed');
  let loading = $state(false);
  let error = $state('');
  let storageWarning = $state('');

  const repos = $derived(data ? filterRepos(data, { view, query, owner, visibility, sort }, seen) : []);
  const owners = $derived(data ? [...new Set([...data.repos, ...data.invitations].map(r => r.full_name.split('/')[0]))].sort((a, b) => a.localeCompare(b)) : []);
  const status = $derived([error, data?.warning, storageWarning, data && !repos.length ? 'No repositories match these filters.' : ''].filter(Boolean).join(' '));
  const date = (iso: string, opts: Intl.DateTimeFormatOptions) => new Date(iso).toLocaleString(undefined, opts);

  async function load() {
    loading = true;
    error = '';
    try {
      data = await invoke<Overview>('overview');
      storageWarning = '';
      try {
        const key = `gitpocket:seen:${data.login}`;
        const saved = localStorage.getItem(key);
        const previous = saved === null ? null : JSON.parse(saved);
        if (previous !== null && (typeof previous !== 'object' || Array.isArray(previous))) throw new Error('Invalid history');
        seen = trackAdditions(previous, data.repos, new Date().toISOString());
        localStorage.setItem(key, JSON.stringify(seen));
      } catch {
        seen = {};
        storageWarning = 'Tracking of new additions is unavailable: local storage is not accessible.';
      }
      if (!owners.includes(owner)) owner = '';
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const repoUrl = (name: string) => `https://github.com/${name.split('/').map(encodeURIComponent).join('/')}`;
</script>

<main class="mx-auto flex max-w-4xl flex-col gap-4 p-6">
  <header class="flex items-start justify-between">
    <div>
      <h1 class="text-2xl font-semibold">GitPocket ↗</h1>
      <p class="text-muted-foreground">Your GitHub repositories, in one place.</p>
    </div>
    <Button onclick={load} disabled={loading}>{loading ? 'Loading…' : 'Refresh'}</Button>
  </header>

  <Input type="search" bind:value={query} placeholder="Search repositories…" aria-label="Search repositories" />

  <Tabs.Root bind:value={view as string}>
    <Tabs.List>
      {#each views as [value, label]}<Tabs.Trigger {value}>{label}</Tabs.Trigger>{/each}
    </Tabs.List>
  </Tabs.Root>

  <div class="grid gap-3 sm:grid-cols-3">
    <Select.Root type="single" bind:value={owner}>
      <Select.Trigger aria-label="Owner">{owner || 'All owners'}</Select.Trigger>
      <Select.Content>
        <Select.Item value="">All owners</Select.Item>
        {#each owners as o}<Select.Item value={o}>{o}</Select.Item>{/each}
      </Select.Content>
    </Select.Root>
    <Select.Root type="single" bind:value={visibility}>
      <Select.Trigger aria-label="Visibility">{visibilities[visibility as keyof typeof visibilities]}</Select.Trigger>
      <Select.Content>
        {#each Object.entries(visibilities) as [v, label]}<Select.Item value={v}>{label}</Select.Item>{/each}
      </Select.Content>
    </Select.Root>
    <Select.Root type="single" bind:value={sort}>
      <Select.Trigger aria-label="Sort by">{sorts[sort as keyof typeof sorts]}</Select.Trigger>
      <Select.Content>
        {#each Object.entries(sorts) as [v, label]}<Select.Item value={v}>{label}</Select.Item>{/each}
      </Select.Content>
    </Select.Root>
  </div>

  <div class="text-muted-foreground flex justify-between text-sm">
    <span>{repos.length} {repos.length === 1 ? 'repository' : 'repositories'}</span>
    {#if data}<span>@{data.login}</span>{/if}
  </div>
  <p role="status" aria-live="polite" class="text-sm {error ? 'text-destructive' : 'text-muted-foreground'}">{status}</p>

  <section class="grid gap-3" aria-label="Repositories">
    {#each repos as repo (repo.id)}
      {@const url = repoUrl(repo.full_name)}
      {@const added = view === 'invitations' ? repo.invited_at : seen[repo.id]}
      <Card.Root>
        <Card.Header>
          <Card.Title><button class="hover:underline" onclick={() => openUrl(url)}>{repo.full_name}</button></Card.Title>
          <Card.Description>{repo.description || 'No description.'}</Card.Description>
          <Card.Action><Badge variant="secondary">{repo.private ? 'Private' : 'Public'}</Badge></Card.Action>
        </Card.Header>
        <Card.Content class="text-muted-foreground flex items-center justify-between text-sm">
          <span>
            {[repo.language, repo.archived ? 'Archived' : '',
              added ? `${view === 'invitations' ? 'Invited' : 'Detected'} on ${date(added, { dateStyle: 'short' })}` : '',
              repo.pushed_at ? `Last push: ${date(repo.pushed_at, { dateStyle: 'short', timeStyle: 'short' })}` : ''].filter(Boolean).join(' · ')}
          </span>
          <Button variant="link" onclick={() => openUrl(`${url}/${view === 'invitations' ? 'invitations' : 'pulls'}`)}>
            {view === 'invitations' ? 'View invitation ↗' : 'Pull requests ↗'}
          </Button>
        </Card.Content>
      </Card.Root>
    {/each}
  </section>
  <footer class="text-muted-foreground text-center text-xs">Read-only · Access depends on your GitHub permissions.</footer>
</main>
