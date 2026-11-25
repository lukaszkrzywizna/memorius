// Feature-Sliced Design layers, highest first: routes and app → pages →
// entities → shared. A layer imports only from lower layers, a slice
// (pages/<name>, entities/<name>) never from a sibling slice, and other
// layers reach a slice only through its index.ts.
/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-circular",
      severity: "error",
      from: {},
      to: { circular: true },
    },
    {
      name: "shared-is-the-bottom",
      severity: "error",
      from: { path: "^src/shared/" },
      to: { path: "^src/(app|routes|pages|entities)/" },
    },
    {
      name: "entities-only-below",
      severity: "error",
      from: { path: "^src/entities/" },
      to: { path: "^src/(app|routes|pages)/" },
    },
    {
      name: "pages-only-below",
      severity: "error",
      from: { path: "^src/pages/" },
      to: { path: "^src/(app|routes)/" },
    },
    {
      name: "no-sibling-slices",
      severity: "error",
      from: { path: "^src/(pages|entities)/([^/]+)/" },
      to: {
        path: "^src/$1/[^/]+/",
        pathNot: "^src/$1/$2/",
      },
    },
    {
      name: "slices-through-their-index",
      severity: "error",
      from: { pathNot: "^src/(pages|entities)/" },
      to: {
        path: "^src/(pages|entities)/[^/]+/",
        pathNot: "^src/(pages|entities)/[^/]+/index\\.ts$",
      },
    },
    {
      name: "pages-reach-entities-through-their-index",
      severity: "error",
      from: { path: "^src/pages/" },
      to: {
        path: "^src/entities/[^/]+/",
        pathNot: "^src/entities/[^/]+/index\\.ts$",
      },
    },
  ],
  options: {
    doNotFollow: { path: "node_modules" },
    exclude: { path: "(^src/shared/api/generated/|routeTree\\.gen\\.ts$)" },
    tsConfig: { fileName: "tsconfig.json" },
    tsPreCompilationDeps: true,
  },
}
