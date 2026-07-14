import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';

export const options = {
  stages: [
    { duration: '1m', target: 20 },
    { duration: '3m', target: 100 },
    { duration: '3m', target: 100 },
    { duration: '1m', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(99)<100', 'p(95)<50'],
    http_req_failed: ['rate<0.01'],
    'http_req_duration{type:write}': ['p(99)<500'],
    'http_req_duration{type:read}': ['p(99)<100'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:5000';

const registeredUsers = new SharedArray('users', () => {
  const users = [];
  for (let i = 0; i < 20; i++) {
    users.push({
      email: `loadtest${i}@example.com`,
      password: 'Test@1234',
    });
  }
  return users;
});

const workshopSlugs = ['rust-basics', 'advanced-rust', 'web-dev-with-rust', 'rust-and-blockchain'];

export function setup() {
  // Register and login users
  const sessions = [];
  for (const user of registeredUsers) {
    const regRes = http.post(`${BASE_URL}/api/v1/auth/register`, JSON.stringify({
      name: `Load Test User ${user.email}`,
      email: user.email,
      password: user.password,
    }), { headers: { 'Content-Type': 'application/json' } });

    const loginRes = http.post(`${BASE_URL}/api/v1/auth/login`, JSON.stringify({
      email: user.email,
      password: user.password,
    }), { headers: { 'Content-Type': 'application/json' } });

    if (loginRes.status === 200) {
      sessions.push({
        cookie: loginRes.headers['Set-Cookie'],
        userId: loginRes.json('data.user.id'),
      });
    }
  }
  return { sessions };
}

export default function (data) {
  const session = data.sessions[Math.floor(Math.random() * data.sessions.length)];
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Cookie': session ? session.cookie : '',
    },
  };

  // GET requests (read-heavy: 80%)
  const read = Math.random() < 0.8;

  if (read) {
    const slug = workshopSlugs[Math.floor(Math.random() * workshopSlugs.length)];
    const endpoints = [
      { url: `${BASE_URL}/api/v1/workshops`, tags: { type: 'read' } },
      { url: `${BASE_URL}/api/v1/categories`, tags: { type: 'read' } },
      { url: `${BASE_URL}/api/v1/workshops/${slug}`, tags: { type: 'read' } },
      { url: `${BASE_URL}/api/v1/enrollments`, tags: { type: 'read' } },
      { url: `${BASE_URL}/api/v1/stats/dashboard`, tags: { type: 'read' } },
    ];
    const ep = endpoints[Math.floor(Math.random() * endpoints.length)];
    const res = http.get(ep.url, { ...params, tags: { ...params.tags, ...ep.tags } });
    check(res, {
      'read status < 500': (r) => r.status < 500,
    });
  } else {
    // POST requests (write-heavy: 20%)
    const endpoints = [
      {
        url: `${BASE_URL}/api/v1/enrollments`,
        body: JSON.stringify({ workshop_slug: workshopSlugs[Math.floor(Math.random() * workshopSlugs.length)] }),
        tags: { type: 'write' },
      },
      {
        url: `${BASE_URL}/api/v1/reviews`,
        body: JSON.stringify({
          workshop_slug: workshopSlugs[Math.floor(Math.random() * workshopSlugs.length)],
          rating: Math.floor(Math.random() * 5) + 1,
          comment: 'Load test review',
        }),
        tags: { type: 'write' },
      },
    ];
    const ep = endpoints[Math.floor(Math.random() * endpoints.length)];
    const res = http.post(ep.url, ep.body, { ...params, tags: { ...params.tags, ...ep.tags } });
    check(res, {
      'write status < 500': (r) => r.status < 500,
    });
  }

  sleep(Math.random() * 2 + 0.5);
}
