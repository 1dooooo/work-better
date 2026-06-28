/**
 * 飞书 API Mock Server
 *
 * 使用 MSW 模拟飞书 API 响应，用于 E2E 测试隔离外部依赖。
 */
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

/**
 * 飞书消息 API mock handlers
 */
const handlers = [
  // 获取消息列表
  http.post("https://open.feishu.cn/open-apis/im/v1/messages", () => {
    return HttpResponse.json({
      code: 0,
      msg: "success",
      data: {
        items: [
          {
            message_id: "mock-msg-001",
            msg_type: "text",
            create_time: Date.now().toString(),
            body: {
              content: JSON.stringify({ text: "飞书消息 1" }),
            },
            sender: {
              sender_id: {
                open_id: "mock-user-001",
              },
              sender_type: "user",
            },
          },
          {
            message_id: "mock-msg-002",
            msg_type: "text",
            create_time: Date.now().toString(),
            body: {
              content: JSON.stringify({ text: "飞书消息 2" }),
            },
            sender: {
              sender_id: {
                open_id: "mock-user-002",
              },
              sender_type: "user",
            },
          },
        ],
        has_more: false,
        page_token: null,
      },
    });
  }),

  // 获取会话信息
  http.get(
    "https://open.feishu.cn/open-apis/im/v1/chats/:chatId",
    ({ params }) => {
      return HttpResponse.json({
        code: 0,
        msg: "success",
        data: {
          chat_id: params.chatId,
          name: "测试会话",
          chat_type: "group",
        },
      });
    },
  ),
];

/**
 * 飞书 API Mock Server 实例
 *
 * 使用方法：
 * - 测试前：feishuServer.listen()
 * - 测试后：feishuServer.close()
 * - 重置 handlers：feishuServer.resetHandlers()
 */
export const feishuServer = setupServer(...handlers);
