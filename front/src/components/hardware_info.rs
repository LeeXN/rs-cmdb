use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::icons::{Cpu, HardDrive, Monitor, Server, Wifi, Zap};
use crate::types::{Hardware, NICStatus, StorageType};
use crate::utils::format::{format_frequency, format_number};
use crate::utils::i18n_helper::t as tr;
use common::entity::hardware::IpmiStatus;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HardwareInfoProps {
    pub hardware: Hardware,
}

#[function_component(HardwareInfo)]
pub fn hardware_info(props: &HardwareInfoProps) -> Html {
    let t = use_trans();
    html! {
        <div class="space-y-6">
            // CPU 卡片
            <Card>
                <CardHeader>
                    <div class="flex items-center space-x-2">
                        <Cpu class="h-5 w-5 text-cyan-500" />
                        <CardTitle>{t.t("hardware.cpu.title")}</CardTitle>
                    </div>
                    <CardDescription>{props.hardware.cpu.model_name.clone()}</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div class="space-y-1">
                            <span class="text-sm text-muted-foreground">{t.t("hardware.label.vendor")}</span>
                            <p class="font-medium">{&props.hardware.cpu.vendor_id}</p>
                        </div>
                        <div class="space-y-1">
                            <span class="text-sm text-muted-foreground">{t.t("hardware.label.frequency")}</span>
                            <p class="font-medium">{format_frequency(props.hardware.cpu.speed as u64 * 1_000_000)}</p>
                        </div>
                        <div class="space-y-1">
                            <span class="text-sm text-muted-foreground">{t.t("hardware.label.cores")}</span>
                            <p class="font-medium">{format_number(props.hardware.cpu.cores as u64)}</p>
                        </div>
                        <div class="space-y-1">
                            <span class="text-sm text-muted-foreground">{t.t("hardware.label.threads")}</span>
                            <p class="font-medium">{format_number(props.hardware.cpu.threads as u64)}</p>
                        </div>
                    </div>
                </CardContent>
            </Card>

            // GPU 卡片
            {render_gpu_card(&props.hardware)}

            // 内存卡片
            {render_memory_card(&props.hardware)}

            // 存储设备卡片
            {render_storage_card(&props.hardware)}

            // 网络设备卡片
            {render_network_card(&props.hardware)}

            // IPMI/BMC 信息卡片
            {render_ipmi_card(&props.hardware)}
        </div>
    }
}

fn render_gpu_card(hardware: &Hardware) -> Html {
    if hardware.gpus.is_empty() {
        return html! {};
    }

    html! {
        <Card>
            <CardHeader>
                <div class="flex items-center space-x-2">
                    <Monitor class="h-5 w-5 text-green-500" />
                    <CardTitle>{tr("hardware.gpu.title")}</CardTitle>
                </div>
                <CardDescription>{hardware.gpus[0].model.clone()}</CardDescription>
            </CardHeader>
            <CardContent>
                <div class="rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>{tr("hardware.label.vendor")}</TableHead>
                                <TableHead>{tr("hardware.label.model")}</TableHead>
                                <TableHead>{tr("hardware.label.device_id")}</TableHead>
                                <TableHead>{tr("hardware.label.driver_version")}</TableHead>
                                <TableHead>{tr("hardware.label.serial_number")}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {
                                hardware.gpus.iter().map(|gpu| {
                                    html! {
                                        <TableRow>
                                            <TableCell>{gpu.vendor.clone()}</TableCell>
                                            <TableCell>{gpu.model.clone()}</TableCell>
                                            <TableCell>{gpu.device_id.clone()}</TableCell>
                                            <TableCell>{gpu.driver_version.clone()}</TableCell>
                                            <TableCell>{gpu.serial_number.clone()}</TableCell>
                                        </TableRow>
                                    }
                                }).collect::<Html>()
                            }
                        </TableBody>
                    </Table>
                </div>
            </CardContent>
        </Card>
    }
}

fn render_memory_card(hardware: &Hardware) -> Html {
    let memory_modules_section = if !hardware.ram.modules.is_empty() {
        html! {
            <div class="mt-6">
                <h4 class="text-sm font-medium mb-4">{tr("hardware.memory.modules_detail")}</h4>
                <div class="rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>{tr("hardware.label.slot")}</TableHead>
                                <TableHead>{tr("hardware.label.capacity")}</TableHead>
                                <TableHead>{tr("hardware.label.frequency")}</TableHead>
                                <TableHead>{tr("hardware.label.type")}</TableHead>
                                <TableHead>{tr("hardware.label.vendor")}</TableHead>
                                <TableHead>{tr("hardware.label.part_number")}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {
                                hardware.ram.modules.iter().map(|module| {
                                    html! {
                                        <TableRow>
                                            <TableCell>{module.locator.clone()}</TableCell>
                                            <TableCell>{format!("{} GB", module.size)}</TableCell>
                                            <TableCell>{format_frequency(module.speed as u64 * 1_000_000)}</TableCell>
                                            <TableCell>{module.memory_type.clone()}</TableCell>
                                            <TableCell>{module.vendor.clone()}</TableCell>
                                            <TableCell>{module.part_number.clone()}</TableCell>
                                        </TableRow>
                                    }
                                }).collect::<Html>()
                            }
                        </TableBody>
                    </Table>
                </div>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <Card>
            <CardHeader>
                <div class="flex items-center space-x-2">
                    <Zap class="h-5 w-5 text-yellow-500" />
                    <CardTitle>{tr("hardware.memory.title")}</CardTitle>
                </div>
                <CardDescription>{format!("{} GB", hardware.ram.total_size)}</CardDescription>
            </CardHeader>
            <CardContent>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="space-y-1">
                        <span class="text-sm text-muted-foreground">{tr("hardware.label.memory_count")}</span>
                        <p class="font-medium">{format!("{} {}", hardware.ram.count, tr("hardware.label.sticks"))}</p>
                    </div>
                    <div class="space-y-1">
                        <span class="text-sm text-muted-foreground">{tr("hardware.label.frequency")}</span>
                        <p class="font-medium">{format_frequency(hardware.ram.speed as u64 * 1_000_000)}</p>
                    </div>
                </div>
                {memory_modules_section}
            </CardContent>
        </Card>
    }
}

fn render_storage_card(hardware: &Hardware) -> Html {
    html! {
        <Card>
            <CardHeader>
                <div class="flex items-center space-x-2">
                    <HardDrive class="h-5 w-5 text-orange-500" />
                    <CardTitle>{tr("hardware.storage.title")}</CardTitle>
                </div>
                <CardDescription>{format!("{} {}", hardware.disks.len(), tr("hardware.label.devices"))}</CardDescription>
            </CardHeader>
            <CardContent class="space-y-4">
                {
                    hardware.disks.iter().map(|disk| {
                        let storage_type_badge = match disk.storage_type {
                            StorageType::SSD => html! { <Badge variant={BadgeVariant::Success}>{"SSD"}</Badge> },
                            StorageType::HDD => html! { <Badge variant={BadgeVariant::Secondary}>{"HDD"}</Badge> },
                            StorageType::NVMe => html! { <Badge variant={BadgeVariant::Default}>{"NVMe"}</Badge> },
                            StorageType::Unknown => html! { <Badge variant={BadgeVariant::Outline}>{tr("hardware.label.unknown")}</Badge> },
                        };

                        html! {
                            <div class="rounded-lg border bg-card text-card-foreground shadow-sm">
                                <details class="group">
                                    <summary class="flex cursor-pointer items-center justify-between p-4 font-medium">
                                        <div class="flex items-center space-x-4">
                                            <span>{&disk.model}</span>
                                            {storage_type_badge}
                                        </div>
                                        <span class="text-sm text-muted-foreground">{format!("{} {}", disk.size, disk.size_unit)}</span>
                                    </summary>
                                    <div class="border-t px-4 py-4">
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                                            <div class="space-y-1">
                                                <span class="text-xs text-muted-foreground">{tr("hardware.label.vendor")}</span>
                                                <p class="text-sm font-medium">{&disk.vendor}</p>
                                            </div>
                                            <div class="space-y-1">
                                                <span class="text-xs text-muted-foreground">{tr("hardware.label.serial_number")}</span>
                                                <p class="text-sm font-medium">{&disk.serial_number}</p>
                                            </div>
                                            <div class="space-y-1">
                                                <span class="text-xs text-muted-foreground">{tr("hardware.label.firmware_version")}</span>
                                                <p class="text-sm font-medium">{&disk.firmware_version}</p>
                                            </div>
                                        </div>

                                        if disk.parted && !disk.partitions.is_empty() {
                                            <div class="mt-4">
                                                <h6 class="text-xs font-semibold uppercase text-muted-foreground mb-2">{tr("hardware.storage.partitions")}</h6>
                                                <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                                                    {
                                                        disk.partitions.iter().map(|partition| {
                                                            html! {
                                                                <div class="rounded bg-muted/50 p-2 text-sm">
                                                                    <span class="font-medium mr-2">{&partition.name}</span>
                                                                    <span class="text-muted-foreground">{format!("{} {}", partition.size, partition.size_unit)}</span>
                                                                </div>
                                                            }
                                                        }).collect::<Html>()
                                                    }
                                                </div>
                                            </div>
                                        }
                                    </div>
                                </details>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </CardContent>
        </Card>
    }
}

fn render_network_card(hardware: &Hardware) -> Html {
    html! {
        <Card>
            <CardHeader>
                <div class="flex items-center space-x-2">
                    <Wifi class="h-5 w-5 text-red-500" />
                    <CardTitle>{tr("hardware.network.title")}</CardTitle>
                </div>
                <CardDescription>{format!("{} {}", hardware.nics.len(), tr("hardware.label.interfaces"))}</CardDescription>
            </CardHeader>
            <CardContent class="space-y-4">
                {
                    hardware.nics.iter().map(|nic| {
                        let status_badge = match nic.status {
                            NICStatus::Up => html! { <Badge variant={BadgeVariant::Success}>{tr("hardware.status.online")}</Badge> },
                            NICStatus::Down => html! { <Badge variant={BadgeVariant::Secondary}>{tr("hardware.status.offline")}</Badge> },
                            NICStatus::Unknown => html! { <Badge variant={BadgeVariant::Warning}>{tr("hardware.label.unknown")}</Badge> },
                        };

                        let type_badge = html! {
                            <Badge variant={BadgeVariant::Outline}>{nic.nic_type.to_string()}</Badge>
                        };

                        let dhcp_badge = if nic.dhcp {
                            html! { <Badge variant={BadgeVariant::Success}>{tr("hardware.status.enabled")}</Badge> }
                        } else {
                            html! { <Badge variant={BadgeVariant::Secondary}>{tr("hardware.status.disabled")}</Badge> }
                        };

                        html! {
                            <div class="rounded-lg border bg-card text-card-foreground shadow-sm">
                                <details class="group">
                                    <summary class="flex cursor-pointer items-center justify-between p-4 font-medium">
                                        <div class="flex items-center space-x-4">
                                            <span>{&nic.name}</span>
                                            <div class="flex space-x-2">
                                                {type_badge}
                                                {status_badge.clone()}
                                            </div>
                                        </div>
                                        <div class="text-right">
                                            <div class="text-sm text-muted-foreground">{&nic.ipv4_address}</div>
                                            <div class="text-xs text-muted-foreground">{format!("{} Mbps", format_number(nic.speed as u64))}</div>
                                        </div>
                                    </summary>
                                    <div class="border-t px-4 py-4">
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.interface_name")}</span><p class="text-sm font-medium">{&nic.name}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.vendor")}</span><p class="text-sm font-medium">{&nic.vendor}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.model")}</span><p class="text-sm font-medium">{&nic.model}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.nic_type")}</span><p class="text-sm font-medium">{nic.nic_type.to_string()}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.pci_slot")}</span><p class="text-sm font-medium">{nic.pci_slot.clone().unwrap_or("N/A".to_string())}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.bandwidth")}</span><p class="text-sm font-medium">{format!("{} Mbps", format_number(nic.speed as u64))}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.status")}</span><div class="mt-1">{status_badge}</div></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.driver")}</span><p class="text-sm font-medium">{&nic.driver}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.firmware_version")}</span><p class="text-sm font-medium">{&nic.firmware_version}</p></div>
                                            if nic.ib_node_type != "Unknown" && !nic.ib_node_type.is_empty() {
                                                <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.ib_node_type")}</span><p class="text-sm font-medium">{&nic.ib_node_type}</p></div>
                                            }
                                        </div>

                                        <div class="border-t my-4"></div>
                                        <h6 class="text-xs font-semibold uppercase text-muted-foreground mb-3">{tr("hardware.network.config")}</h6>

                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.mac_address")}</span><p class="text-sm font-medium">{&nic.mac_address}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.dhcp")}</span><div class="mt-1">{dhcp_badge}</div></div>
                                        </div>

                                        <h6 class="text-xs font-semibold uppercase text-muted-foreground mb-2">{tr("hardware.network.ipv4_config")}</h6>
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.ip_address")}</span><p class="text-sm font-medium">{&nic.ipv4_address}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.subnet_mask")}</span><p class="text-sm font-medium">{&nic.ipv4_subnet_mask}</p></div>
                                            <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.gateway")}</span><p class="text-sm font-medium">{&nic.ipv4_gateway}</p></div>
                                        </div>

                                        if !nic.ipv6_address.is_empty() && nic.ipv6_address != "N/A" {
                                            <div class="mt-4">
                                                <h6 class="text-xs font-semibold uppercase text-muted-foreground mb-2">{tr("hardware.network.ipv6_config")}</h6>
                                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                                    <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.ip_address")}</span><p class="text-sm font-medium">{&nic.ipv6_address}</p></div>
                                                    <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.subnet_mask")}</span><p class="text-sm font-medium">{&nic.ipv6_subnet_mask}</p></div>
                                                    <div class="space-y-1"><span class="text-xs text-muted-foreground">{tr("hardware.label.gateway")}</span><p class="text-sm font-medium">{&nic.ipv6_gateway}</p></div>
                                                </div>
                                            </div>
                                        }

                                        if !nic.bonding_slaves.is_empty() {
                                            <div class="mt-4">
                                                <h6 class="text-xs font-semibold uppercase text-muted-foreground mb-2">{tr("hardware.network.bonding_slaves")}</h6>
                                                <div class="flex flex-wrap gap-2">
                                                    {
                                                        nic.bonding_slaves.iter().map(|slave| {
                                                            html! {
                                                                <div class="rounded bg-muted/50 px-2 py-1 text-sm font-medium">
                                                                    {slave}
                                                                </div>
                                                            }
                                                        }).collect::<Html>()
                                                    }
                                                </div>
                                            </div>
                                        }
                                    </div>
                                </details>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </CardContent>
        </Card>
    }
}

fn render_ipmi_card(hardware: &Hardware) -> Html {
    if let Some(ipmi) = &hardware.ipmi {
        let (status_text, card_content) = match &ipmi.status {
            IpmiStatus::Available => {
                let users_section = if !ipmi.users.is_empty() {
                    html! {
                        <div class="mt-6">
                            <h4 class="text-sm font-medium mb-4">{tr("hardware.ipmi.users")}</h4>
                            <div class="rounded-md border">
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>{tr("hardware.label.user_id")}</TableHead>
                                            <TableHead>{tr("hardware.label.username")}</TableHead>
                                            <TableHead>{tr("hardware.label.status")}</TableHead>
                                            <TableHead>{tr("hardware.label.privilege")}</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {
                                            ipmi.users.iter().map(|user| {
                                                let status_badge = if user.enabled {
                                                    html! { <Badge variant={BadgeVariant::Success}>{tr("hardware.status.enabled")}</Badge> }
                                                } else {
                                                    html! { <Badge variant={BadgeVariant::Secondary}>{tr("hardware.status.disabled")}</Badge> }
                                                };

                                                let privilege_text = match user.privilege_level {
                                                    1 => tr("hardware.ipmi.privilege_callback"),
                                                    2 => tr("hardware.ipmi.privilege_user"),
                                                    3 => tr("hardware.ipmi.privilege_operator"),
                                                    4 => tr("hardware.ipmi.privilege_admin"),
                                                    15 => tr("hardware.ipmi.privilege_no_access"),
                                                    _ => tr("hardware.label.unknown"),
                                                };

                                                html! {
                                                    <TableRow>
                                                        <TableCell>{user.user_id.to_string()}</TableCell>
                                                        <TableCell>{user.username.clone()}</TableCell>
                                                        <TableCell>{status_badge}</TableCell>
                                                        <TableCell>{privilege_text}</TableCell>
                                                    </TableRow>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </TableBody>
                                </Table>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                };

                (
                    tr("hardware.ipmi.status_available"),
                    html! {
                        <>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1"><span class="text-sm text-muted-foreground">{tr("hardware.label.ip_address")}</span><p class="font-medium">{ipmi.ip_address.as_ref().cloned().unwrap_or_else(|| tr("hardware.label.unknown"))}</p></div>
                                <div class="space-y-1"><span class="text-sm text-muted-foreground">{tr("hardware.label.mac_address")}</span><p class="font-medium">{ipmi.mac_address.as_ref().cloned().unwrap_or_else(|| tr("hardware.label.unknown"))}</p></div>
                                <div class="space-y-1"><span class="text-sm text-muted-foreground">{tr("hardware.label.subnet_mask")}</span><p class="font-medium">{ipmi.subnet_mask.as_ref().cloned().unwrap_or_else(|| tr("hardware.label.unknown"))}</p></div>
                                <div class="space-y-1"><span class="text-sm text-muted-foreground">{tr("hardware.label.gateway")}</span><p class="font-medium">{ipmi.gateway.as_ref().cloned().unwrap_or_else(|| tr("hardware.label.unknown"))}</p></div>
                                <div class="space-y-1"><span class="text-sm text-muted-foreground">{tr("hardware.label.channel")}</span><p class="font-medium">{ipmi.channel.to_string()}</p></div>
                                <div class="space-y-1"><span class="text-sm text-muted-foreground">{tr("hardware.label.firmware_version")}</span><p class="font-medium">{ipmi.firmware_version.as_ref().cloned().unwrap_or_else(|| tr("hardware.label.unknown"))}</p></div>
                            </div>
                            {users_section}
                        </>
                    },
                )
            }
            IpmiStatus::Error(msg) => (
                tr("hardware.ipmi.status_error"),
                html! {
                    <div class="rounded-md bg-yellow-500/10 p-4 text-yellow-500 border border-yellow-500/20">
                        <strong>{tr("common.error_prefix")}</strong>{msg}
                    </div>
                },
            ),
            IpmiStatus::NotConfigured => (
                tr("hardware.ipmi.status_not_configured"),
                html! {
                    <div class="rounded-md bg-blue-500/10 p-4 text-blue-500 border border-blue-500/20">
                        {tr("hardware.ipmi.not_configured")}
                    </div>
                },
            ),
            IpmiStatus::NotAvailable => (
                tr("hardware.ipmi.status_not_available"),
                html! {
                    <div class="rounded-md bg-blue-500/10 p-4 text-blue-500 border border-blue-500/20">
                        {tr("hardware.ipmi.not_available")}
                    </div>
                },
            ),
            IpmiStatus::AccessDenied => (
                tr("hardware.ipmi.status_access_denied"),
                html! {
                    <div class="rounded-md bg-yellow-500/10 p-4 text-yellow-500 border border-yellow-500/20">
                        {tr("hardware.ipmi.access_denied")}
                    </div>
                },
            ),
        };

        html! {
            <Card>
                <CardHeader>
                    <div class="flex items-center space-x-2">
                        <Server class="h-5 w-5 text-slate-400" />
                        <CardTitle>{"IPMI/BMC"}</CardTitle>
                    </div>
                    <CardDescription>{status_text}</CardDescription>
                </CardHeader>
                <CardContent>
                    {card_content}
                </CardContent>
            </Card>
        }
    } else {
        html! {}
    }
}
