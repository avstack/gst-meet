initSidebarItems({"constant":[["NICE_AGENT_MAX_REMOTE_CANDIDATES",""],["NICE_AGENT_OPTION_CONSENT_FRESHNESS",""],["NICE_AGENT_OPTION_ICE_TRICKLE",""],["NICE_AGENT_OPTION_LITE_MODE",""],["NICE_AGENT_OPTION_REGULAR_NOMINATION",""],["NICE_AGENT_OPTION_RELIABLE",""],["NICE_AGENT_OPTION_SUPPORT_RENOMINATION",""],["NICE_CANDIDATE_MAX_FOUNDATION",""],["NICE_CANDIDATE_MAX_LOCAL_ADDRESSES",""],["NICE_CANDIDATE_MAX_TURN_SERVERS",""],["NICE_CANDIDATE_TRANSPORT_TCP_ACTIVE",""],["NICE_CANDIDATE_TRANSPORT_TCP_PASSIVE",""],["NICE_CANDIDATE_TRANSPORT_TCP_SO",""],["NICE_CANDIDATE_TRANSPORT_UDP",""],["NICE_CANDIDATE_TYPE_HOST",""],["NICE_CANDIDATE_TYPE_PEER_REFLEXIVE",""],["NICE_CANDIDATE_TYPE_RELAYED",""],["NICE_CANDIDATE_TYPE_SERVER_REFLEXIVE",""],["NICE_COMPATIBILITY_DRAFT19",""],["NICE_COMPATIBILITY_GOOGLE",""],["NICE_COMPATIBILITY_LAST",""],["NICE_COMPATIBILITY_MSN",""],["NICE_COMPATIBILITY_OC2007",""],["NICE_COMPATIBILITY_OC2007R2",""],["NICE_COMPATIBILITY_RFC5245",""],["NICE_COMPATIBILITY_WLM2009",""],["NICE_COMPONENT_STATE_CONNECTED",""],["NICE_COMPONENT_STATE_CONNECTING",""],["NICE_COMPONENT_STATE_DISCONNECTED",""],["NICE_COMPONENT_STATE_FAILED",""],["NICE_COMPONENT_STATE_GATHERING",""],["NICE_COMPONENT_STATE_LAST",""],["NICE_COMPONENT_STATE_READY",""],["NICE_COMPONENT_TYPE_RTCP",""],["NICE_COMPONENT_TYPE_RTP",""],["NICE_NOMINATION_MODE_AGGRESSIVE",""],["NICE_NOMINATION_MODE_REGULAR",""],["NICE_PROXY_TYPE_HTTP",""],["NICE_PROXY_TYPE_LAST",""],["NICE_PROXY_TYPE_NONE",""],["NICE_PROXY_TYPE_SOCKS5",""],["NICE_RELAY_TYPE_TURN_TCP",""],["NICE_RELAY_TYPE_TURN_TLS",""],["NICE_RELAY_TYPE_TURN_UDP",""],["PSEUDO_TCP_CLOSED",""],["PSEUDO_TCP_CLOSE_WAIT",""],["PSEUDO_TCP_CLOSING",""],["PSEUDO_TCP_DEBUG_NONE",""],["PSEUDO_TCP_DEBUG_NORMAL",""],["PSEUDO_TCP_DEBUG_VERBOSE",""],["PSEUDO_TCP_ESTABLISHED",""],["PSEUDO_TCP_FIN_WAIT_1",""],["PSEUDO_TCP_FIN_WAIT_2",""],["PSEUDO_TCP_LAST_ACK",""],["PSEUDO_TCP_LISTEN",""],["PSEUDO_TCP_SHUTDOWN_RD",""],["PSEUDO_TCP_SHUTDOWN_RDWR",""],["PSEUDO_TCP_SHUTDOWN_WR",""],["PSEUDO_TCP_SYN_RECEIVED",""],["PSEUDO_TCP_SYN_SENT",""],["PSEUDO_TCP_TIME_WAIT",""],["WR_FAIL",""],["WR_SUCCESS",""],["WR_TOO_LARGE",""]],"fn":[["nice_address_copy_to_sockaddr",""],["nice_address_dup",""],["nice_address_equal",""],["nice_address_equal_no_port",""],["nice_address_free",""],["nice_address_get_port",""],["nice_address_init",""],["nice_address_ip_version",""],["nice_address_is_private",""],["nice_address_is_valid",""],["nice_address_new",""],["nice_address_set_from_sockaddr",""],["nice_address_set_from_string",""],["nice_address_set_ipv4",""],["nice_address_set_ipv6",""],["nice_address_set_port",""],["nice_address_to_string",""],["nice_agent_add_local_address",""],["nice_agent_add_stream",""],["nice_agent_attach_recv",""],["nice_agent_close_async",""],["nice_agent_forget_relays",""],["nice_agent_gather_candidates",""],["nice_agent_generate_local_candidate_sdp",""],["nice_agent_generate_local_sdp",""],["nice_agent_generate_local_stream_sdp",""],["nice_agent_get_component_state",""],["nice_agent_get_default_local_candidate",""],["nice_agent_get_io_stream",""],["nice_agent_get_local_candidates",""],["nice_agent_get_local_credentials",""],["nice_agent_get_remote_candidates",""],["nice_agent_get_selected_pair",""],["nice_agent_get_selected_socket",""],["nice_agent_get_stream_name",""],["nice_agent_get_type",""],["nice_agent_new",""],["nice_agent_new_full",""],["nice_agent_new_reliable",""],["nice_agent_parse_remote_candidate_sdp",""],["nice_agent_parse_remote_sdp",""],["nice_agent_parse_remote_stream_sdp",""],["nice_agent_peer_candidate_gathering_done",""],["nice_agent_recv",""],["nice_agent_recv_messages",""],["nice_agent_recv_messages_nonblocking",""],["nice_agent_recv_nonblocking",""],["nice_agent_remove_stream",""],["nice_agent_restart",""],["nice_agent_restart_stream",""],["nice_agent_send",""],["nice_agent_send_messages_nonblocking",""],["nice_agent_set_local_credentials",""],["nice_agent_set_port_range",""],["nice_agent_set_relay_info",""],["nice_agent_set_remote_candidates",""],["nice_agent_set_remote_credentials",""],["nice_agent_set_selected_pair",""],["nice_agent_set_selected_remote_candidate",""],["nice_agent_set_software",""],["nice_agent_set_stream_name",""],["nice_agent_set_stream_tos",""],["nice_candidate_copy",""],["nice_candidate_equal_target",""],["nice_candidate_free",""],["nice_candidate_get_type",""],["nice_candidate_new",""],["nice_component_state_to_string",""],["nice_debug_disable",""],["nice_debug_enable",""],["nice_interfaces_get_ip_for_interface",""],["nice_interfaces_get_local_interfaces",""],["nice_interfaces_get_local_ips",""],["pseudo_tcp_set_debug_level",""],["pseudo_tcp_socket_can_send",""],["pseudo_tcp_socket_close",""],["pseudo_tcp_socket_connect",""],["pseudo_tcp_socket_get_available_bytes",""],["pseudo_tcp_socket_get_available_send_space",""],["pseudo_tcp_socket_get_error",""],["pseudo_tcp_socket_get_next_clock",""],["pseudo_tcp_socket_get_type",""],["pseudo_tcp_socket_is_closed",""],["pseudo_tcp_socket_is_closed_remotely",""],["pseudo_tcp_socket_new",""],["pseudo_tcp_socket_notify_clock",""],["pseudo_tcp_socket_notify_message",""],["pseudo_tcp_socket_notify_mtu",""],["pseudo_tcp_socket_notify_packet",""],["pseudo_tcp_socket_recv",""],["pseudo_tcp_socket_send",""],["pseudo_tcp_socket_set_time",""],["pseudo_tcp_socket_shutdown",""]],"struct":[["NiceAddress",""],["NiceAgent",""],["NiceAgentClass",""],["NiceCandidate",""],["NiceInputMessage",""],["NiceOutputMessage",""],["PseudoTcpCallbacks",""],["PseudoTcpSocket",""],["_PseudoTcpSocketClass",""]],"type":[["NiceAgentOption",""],["NiceAgentRecvFunc",""],["NiceCandidateTransport",""],["NiceCandidateType",""],["NiceCompatibility",""],["NiceComponentState",""],["NiceComponentType",""],["NiceNominationMode",""],["NiceProxyType",""],["NiceRelayType",""],["PseudoTcpDebugLevel",""],["PseudoTcpShutdown",""],["PseudoTcpSocketClass",""],["PseudoTcpState",""],["PseudoTcpWriteResult",""]],"union":[["NiceAddress_s",""]]});