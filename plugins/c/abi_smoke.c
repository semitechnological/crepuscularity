#include "../../crates/crepuscularity-abi/include/crepuscularity_abi.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int seen_event = 0;

static void capture_event(const char *event_json, void *userdata) {
    (void)userdata;
    if (event_json != NULL && strstr(event_json, "\"handler\":\"bind:count:2\"") != NULL) {
        seen_event = 1;
    }
}

static int fail(CrepusSession *session) {
    char *error = crepus_session_take_last_error(session);
    if (error != NULL) {
        fputs(error, stderr);
        crepus_string_free(error);
    }
    crepus_session_free(session);
    return 1;
}

int main(void) {
    CrepusSession *session = crepus_session_new();
    if (session == NULL) {
        return 1;
    }
    if (crepus_session_set_template_string(session, "input bind=count\nspan\n  \"Count {count}\"", NULL) != 0) {
        return fail(session);
    }
    if (crepus_session_set_context_json(session, "{\"count\":\"1\"}") != 0) {
        return fail(session);
    }
    if (crepus_session_set_event_callback(session, capture_event, NULL) != 0) {
        return fail(session);
    }
    char *first = crepus_session_render_ir_json(session);
    if (first == NULL || strstr(first, "Count 1") == NULL) {
        if (first != NULL) {
            crepus_string_free(first);
        }
        return fail(session);
    }
    crepus_string_free(first);
    char *event = crepus_session_dispatch_event_json(session, "{\"handler\":\"bind:count:2\"}");
    if (event == NULL || strstr(event, "Count 2") == NULL || !seen_event) {
        if (event != NULL) {
            crepus_string_free(event);
        }
        return fail(session);
    }
    crepus_string_free(event);
    crepus_session_free(session);
    return 0;
}
