import type { ActionMessage, AgentToRendererMessage } from "@a2ui/web_core/v1_0";

export const STREAM_SURFACE_ID = "stream";
export const BASIC_CATALOG_ID = "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";

export const streamMessages: readonly AgentToRendererMessage[] = [
  {
    version: "1.0",
    createSurface: {
      surfaceId: STREAM_SURFACE_ID,
      catalogId: BASIC_CATALOG_ID,
    },
  },
  {
    version: "1.0",
    updateDataModel: {
      surfaceId: STREAM_SURFACE_ID,
      path: "/trip",
      value: {
        from: "Berlin",
        to: "San Francisco",
        passengers: 1,
        total: 455,
      },
    },
  },
  {
    version: "1.0",
    updateComponents: {
      surfaceId: STREAM_SURFACE_ID,
      components: [
        {
          id: "root",
          component: "Column",
          children: ["heading", "summary", "description", "total_row", "actions"],
        },
      ],
    },
  },
  {
    version: "1.0",
    updateComponents: {
      surfaceId: STREAM_SURFACE_ID,
      components: [{ id: "heading", component: "Text", text: "Trip planner" }],
    },
  },
  {
    version: "1.0",
    updateComponents: {
      surfaceId: STREAM_SURFACE_ID,
      components: [
        {
          id: "summary",
          component: "Text",
          text: {
            "@call": "formatString",
            catalogId: BASIC_CATALOG_ID,
            args: {
              value: `Route: \${/trip/from} to \${/trip/to} | Passengers: \${/trip/passengers}`,
            },
          },
        },
      ],
    },
  },
  {
    version: "1.0",
    updateComponents: {
      surfaceId: STREAM_SURFACE_ID,
      components: [
        {
          id: "description",
          component: "Text",
          text: "Nonstop flight | 11h 30m | Premium economy",
          variant: "caption",
        },
        {
          id: "total_row",
          component: "Row",
          children: ["total_label", "total_value"],
        },
        { id: "total_label", component: "Text", text: "Total" },
        {
          id: "total_value",
          component: "Text",
          text: {
            "@call": "formatCurrency",
            catalogId: BASIC_CATALOG_ID,
            args: {
              value: { "@path": "/trip/total" },
              currency: "USD",
              decimals: 0,
            },
          },
        },
      ],
    },
  },
  {
    version: "1.0",
    updateComponents: {
      surfaceId: STREAM_SURFACE_ID,
      components: [
        {
          id: "actions",
          component: "Row",
          children: ["add_passenger", "documentation"],
        },
        {
          id: "add_passenger",
          component: "Button",
          child: "add_passenger_label",
          action: {
            event: {
              name: "add_passenger",
              context: { passengers: { "@path": "/trip/passengers" } },
            },
          },
          variant: "primary",
        },
        { id: "add_passenger_label", component: "Text", text: "Add passenger" },
        {
          id: "documentation",
          component: "Button",
          child: "documentation_label",
          action: {
            functionCall: {
              "@call": "openUrl",
              catalogId: BASIC_CATALOG_ID,
              args: { url: "https://a2ui.org/" },
            },
          },
          variant: "borderless",
        },
        {
          id: "documentation_label",
          component: "Text",
          text: "A2UI documentation",
        },
      ],
    },
  },
];

export function handleStreamAction(message: ActionMessage): AgentToRendererMessage[] {
  if (message.action.name !== "add_passenger") {
    throw new Error(`Unsupported stream action: ${message.action.name}`);
  }

  const { passengers: currentPassengers } = message.action.context;
  if (
    typeof currentPassengers !== "number" ||
    !Number.isSafeInteger(currentPassengers) ||
    currentPassengers < 1
  ) {
    throw new Error("add_passenger requires a positive integer context.passengers");
  }

  const passengers = currentPassengers + 1;
  return [
    {
      version: "1.0",
      updateDataModel: {
        surfaceId: STREAM_SURFACE_ID,
        path: "/trip/passengers",
        value: passengers,
      },
    },
    {
      version: "1.0",
      updateDataModel: {
        surfaceId: STREAM_SURFACE_ID,
        path: "/trip/total",
        value: 455 * passengers,
      },
    },
  ];
}
