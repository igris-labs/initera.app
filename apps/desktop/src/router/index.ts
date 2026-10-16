import { createRouter, createWebHashHistory } from 'vue-router';
import IssuesPage from '../pages/issues/index.vue';

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/issues' },
    {
      path: '/issues',
      name: 'issues',
      component: IssuesPage,
      children: [
        { path: ':id', name: 'issue', component: () => import('../pages/issues/[id].vue') },
      ],
    },
    {
      path: '/issues/:id/full',
      name: 'issue-full',
      component: () => import('../pages/issues/[id]/full.vue'),
    },
    { path: '/triage', name: 'triage', component: () => import('../pages/triage/index.vue') },
    { path: '/archives', name: 'archives', component: () => import('../pages/archives/index.vue') },
    { path: '/cycles', name: 'cycles', component: () => import('../pages/cycles/index.vue') },
    { path: '/time', name: 'time', component: () => import('../pages/time/index.vue') },
    {
      path: '/projects',
      name: 'projects',
      component: () => import('../pages/projects/index.vue'),
    },
    {
      path: '/projects/:project',
      name: 'project-overview',
      component: () => import('../pages/projects/[project]/index.vue'),
    },
    {
      path: '/projects/:project/issues',
      name: 'project-issues',
      component: () => import('../pages/projects/[project]/issues.vue'),
    },
    {
      path: '/projects/:project/structure',
      name: 'project-structure',
      component: () => import('../pages/projects/[project]/structure.vue'),
    },
    { path: '/views', name: 'views', component: () => import('../pages/views/index.vue') },
    {
      path: '/views/:viewId',
      name: 'saved-view',
      component: () => import('../pages/views/[id].vue'),
    },
    { path: '/settings', name: 'settings', component: () => import('../pages/settings/index.vue') },
    { path: '/:pathMatch(.*)*', redirect: '/issues' },
  ],
});
