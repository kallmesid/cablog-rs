use leptos::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use gloo_net::http::Request;
use wasm_bindgen::UnwrapThrowExt;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Booking {
    pub id: String,
    pub employee_name: String,
    pub department: String,
    pub pickup_location: String,
    pub drop_location: String,
    pub date_time: String,
    pub vendor_name: String,
    pub vendor_email: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SentEmail {
    pub id: String,
    pub booking_id: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub sent_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OdometerCount {
    pub count: u32,
}

#[component]
pub fn App() -> impl IntoView {
    // App Data Signals
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (sent_emails, set_sent_emails) = create_signal(Vec::<SentEmail>::new());
    let (selected_email, set_selected_email) = create_signal(None::<SentEmail>);
    let (today_count, set_today_count) = create_signal(0u32);

    // Form inputs Signals
    let (employee_name, set_employee_name) = create_signal(String::new());
    let (department, set_department) = create_signal(String::from("Operations"));
    let (pickup_location, set_pickup_location) = create_signal(String::new());
    let (drop_location, set_drop_location) = create_signal(String::new());
    let (date_time, set_date_time) = create_signal(String::new());
    let (vendor_index, set_vendor_index) = create_signal(0usize);

    // UI Feedback Signals
    let (is_submitting, set_is_submitting) = create_signal(false);
    let (validation_err, set_validation_err) = create_signal(None::<String>);
    let (success_msg, set_success_msg) = create_signal(None::<String>);

    let vendors: &[(&'static str, &'static str)] = &[
        ("Metro Cab Link", "dispatch@metrocablink.com"),
        ("Swift Fleet Logistics", "bookings@swiftfleet.net"),
        ("Horizon Shuttles", "dispatch@horizonshuttles.co"),
        ("Beacon Town Cars", "reservation@beacontowncars.com"),
    ];

    let departments: &[&'static str] = &[
        "Operations",
        "Engineering",
        "Legal & Finance",
        "HR & Recruitment",
        "Product Management",
        "Executive Suite",
    ];

    // Fetch data function
    let fetch_data = move || {
        spawn_local(async move {
            // Fetch Bookings
            if let Ok(resp) = Request::get("http://localhost:8095/api/bookings").send().await {
                if let Ok(data) = resp.json::<Vec<Booking>>().await {
                    set_bookings.set(data);
                }
            }

            // Fetch Emails
            if let Ok(resp) = Request::get("http://localhost:8095/api/sent-emails").send().await {
                if let Ok(data) = resp.json::<Vec<SentEmail>>().await {
                    set_sent_emails.set(data.clone());
                    if !data.is_empty() && selected_email.get_untracked().is_none() {
                        set_selected_email.set(Some(data[0].clone()));
                    }
                }
            }

            // Fetch Today Count
            if let Ok(resp) = Request::get("http://localhost:8095/api/bookings/today-count").send().await {
                if let Ok(data) = resp.json::<OdometerCount>().await {
                    set_today_count.set(data.count);
                }
            }
        });
    };

    // Initial load
    create_effect(move |_| {
        fetch_data();
    });

    // Handle Form Submit
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_validation_err.set(None);
        set_success_msg.set(None);

        let name = employee_name.get();
        let pickup = pickup_location.get();
        let drop = drop_location.get();
        let dt = date_time.get();

        if name.trim().is_empty() {
            set_validation_err.set(Some("Employee name is required".to_string()));
            return;
        }
        if pickup.trim().is_empty() {
            set_validation_err.set(Some("Pickup location is required".to_string()));
            return;
        }
        if drop.trim().is_empty() {
            set_validation_err.set(Some("Drop-off destination is required".to_string()));
            return;
        }
        if dt.trim().is_empty() {
            set_validation_err.set(Some("Pickup date & time are required".to_string()));
            return;
        }

        set_is_submitting.set(true);

        let v_idx = vendor_index.get();
        let (vendor_name, vendor_email) = vendors[v_idx];

        let payload = json!({
            "employeeName": name.trim(),
            "department": department.get(),
            "pickupLocation": pickup.trim(),
            "dropLocation": drop.trim(),
            "dateTime": dt,
            "vendorName": vendor_name,
            "vendorEmail": vendor_email
        });

        spawn_local(async move {
            let resp = Request::post("http://localhost:8095/api/bookings")
                .header("Content-Type", "application/json")
                .body(payload.to_string())
                .unwrap()
                .send()
                .await;

            set_is_submitting.set(false);

            match resp {
                Ok(r) if r.status() == 201 => {
                    set_success_msg.set(Some("Ticket logged & dispatched successfully!".to_string()));
                    set_employee_name.set(String::new());
                    set_pickup_location.set(String::new());
                    set_drop_location.set(String::new());
                    set_date_time.set(String::new());
                    fetch_data();
                }
                _ => {
                    set_validation_err.set(Some("Failed to dispatch ticket. Check backend connection.".to_string()));
                }
            }
        });
    };

    // Action to send/mark as sent manually
    let mark_as_sent = move |id: String| {
        spawn_local(async move {
            let endpoint = format!("http://localhost:8095/api/bookings/{}/send", id);
            let resp = Request::post(&endpoint).send().await;
            if resp.is_ok() {
                fetch_data();
            }
        });
    };

    // Render mechanical odometer count
    let render_odometer = move || {
        let count_str = format!("{:04}", today_count.get());
        view! {
            <div class="flex items-center gap-1 select-none">
                <span class="hidden md:inline font-display text-[#8A8478] tracking-widest text-xs uppercase mr-2 font-semibold">
                    "LOGGED TODAY"
                </span>
                <div class="flex bg-[#1F2421] p-1.5 border-2 border-[#8A8478]/40 rounded-md shadow-lg">
                    {count_str.chars().map(|c| {
                        view! {
                            <div class="w-7 h-10 bg-black text-white font-mono font-bold text-xl flex items-center justify-center border-r border-gray-800 last:border-0 relative overflow-hidden">
                                <div class="absolute inset-x-0 top-0 h-1 bg-gradient-to-b from-black/80 to-transparent" />
                                <div class="absolute inset-x-0 bottom-0 h-1 bg-gradient-to-t from-black/80 to-transparent" />
                                <div class="absolute inset-x-0 top-1/2 h-[1px] bg-white/5" />
                                <span class="text-[#F2EFE6] relative z-10">{c}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        }
    };

    view! {
        <div class="min-h-screen bg-[#F2EFE6] text-[#1F2421] font-sans antialiased pb-12">
            
            // Header / Brand Title
            <header class="border-b-4 border-[#1F2421] bg-[#1F2421] text-[#F2EFE6] py-4 px-6 md:px-12 shadow-md">
                <div class="max-w-7xl mx-auto flex justify-between items-center">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-[#C8553D] flex items-center justify-center rounded border-2 border-[#F2EFE6]">
                            <span class="font-display font-black text-2xl tracking-tighter text-[#F2EFE6]">"C"</span>
                        </div>
                        <div>
                            <h1 class="font-display text-2xl md:text-3xl font-extrabold tracking-tight uppercase leading-none">
                                "CAB LOGGER"
                            </h1>
                            <span class="font-mono text-[10px] text-[#8A8478] tracking-widest uppercase">
                                "RUST + LEPTOS DISPATCH BOARD"
                            </span>
                        </div>
                    </div>
                    {render_odometer}
                </div>
            </header>

            <div class="h-2 bg-[#8A8478] opacity-30 w-full" />

            // Grid Layout
            <main class="max-w-7xl mx-auto px-4 md:px-8 mt-8 grid grid-cols-1 lg:grid-cols-12 gap-8">
                
                // Left Panel: Form & Email Preview
                <div class="lg:col-span-5 flex flex-col gap-8">
                    
                    // Dispatch Ticket Order Form
                    <div class="bg-white border-2 border-[#1F2421] relative shadow-lg overflow-hidden">
                        <div class="h-1.5 bg-[#C8553D] w-full" />
                        
                        <div class="absolute -left-2 top-24 w-4 h-4 rounded-full bg-[#F2EFE6] border-r-2 border-[#1F2421]" />
                        <div class="absolute -right-2 top-24 w-4 h-4 rounded-full bg-[#F2EFE6] border-l-2 border-[#1F2421]" />

                        <div class="p-6 md:p-8">
                            <div class="flex justify-between items-start border-b-2 border-dashed border-[#8A8478]/40 pb-4 mb-6">
                                <div>
                                    <h2 class="font-display text-2xl font-bold tracking-tight uppercase text-[#1F2421]">
                                        "DISPATCH TICKET"
                                    </h2>
                                    <p class="text-xs text-[#8A8478] font-mono mt-0.5">
                                        "REQUISITION SLIP • RUST CORE ENGINE"
                                    </p>
                                </div>
                                <div class="font-mono text-xs bg-[#1F2421]/5 text-[#1F2421] px-2 py-1 border border-[#1F2421]/20 rounded uppercase">
                                    "No. 104-B"
                                </div>
                            </div>

                            // Validations & Success Alerts
                            {move || validation_err.get().map(|err| view! {
                                <div class="mb-4 bg-[#C8553D]/10 border border-[#C8553D] text-[#C8553D] p-3 text-xs font-mono">
                                    {err}
                                </div>
                            })}

                            {move || success_msg.get().map(|msg| view! {
                                <div class="mb-4 bg-[#5C7A6E]/15 border border-[#5C7A6E] text-[#5C7A6E] p-3 text-xs font-mono flex items-center gap-2">
                                    <div class="w-2 h-2 rounded-full bg-[#5C7A6E] animate-pulse" />
                                    {msg}
                                </div>
                            })}

                            <form on:submit=on_submit class="space-y-4">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div>
                                        <label class="block text-xs font-mono font-bold text-[#1F2421] uppercase mb-1">
                                            "Passenger Name"
                                        </label>
                                        <input 
                                            type="text" 
                                            placeholder="e.g. Jameson Patel"
                                            prop:value=employee_name
                                            on:input=move |ev| set_employee_name.set(event_target_value(&ev))
                                            class="w-full bg-[#F2EFE6]/30 border border-[#8A8478] focus:border-[#1F2421] focus:ring-0 focus:outline-none px-3 py-2 text-sm text-[#1F2421] font-mono"
                                        />
                                    </div>

                                    <div>
                                        <label class="block text-xs font-mono font-bold text-[#1F2421] uppercase mb-1">
                                            "Department"
                                        </label>
                                        <select 
                                            on:change=move |ev| set_department.set(event_target_value(&ev))
                                            class="w-full bg-[#F2EFE6]/30 border border-[#8A8478] focus:border-[#1F2421] focus:ring-0 focus:outline-none px-3 py-2 text-sm text-[#1F2421] font-mono"
                                        >
                                            {departments.iter().map(|dept| view! {
                                                <option value=*dept selected={*dept == department.get_untracked()}>{*dept}</option>
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    </div>
                                </div>

                                <div>
                                    <label class="block text-xs font-mono font-bold text-[#1F2421] uppercase mb-1">
                                        "Pickup Point"
                                    </label>
                                    <input 
                                        type="text" 
                                        placeholder="e.g. West Labs Loading Dock"
                                        prop:value=pickup_location
                                        on:input=move |ev| set_pickup_location.set(event_target_value(&ev))
                                        class="w-full bg-[#F2EFE6]/30 border border-[#8A8478] focus:border-[#1F2421] focus:ring-0 focus:outline-none px-3 py-2 text-sm text-[#1F2421] font-mono"
                                    />
                                </div>

                                <div>
                                    <label class="block text-xs font-mono font-bold text-[#1F2421] uppercase mb-1">
                                        "Drop-off Destination"
                                    </label>
                                    <input 
                                        type="text" 
                                        placeholder="e.g. SFO Airport Terminal 1"
                                        prop:value=drop_location
                                        on:input=move |ev| set_drop_location.set(event_target_value(&ev))
                                        class="w-full bg-[#F2EFE6]/30 border border-[#8A8478] focus:border-[#1F2421] focus:ring-0 focus:outline-none px-3 py-2 text-sm text-[#1F2421] font-mono"
                                    />
                                </div>

                                <div>
                                    <label class="block text-xs font-mono font-bold text-[#1F2421] uppercase mb-1">
                                        "Pickup Date & Time"
                                    </label>
                                    <input 
                                        type="datetime-local" 
                                        prop:value=date_time
                                        on:input=move |ev| set_date_time.set(event_target_value(&ev))
                                        class="w-full bg-[#F2EFE6]/30 border border-[#8A8478] focus:border-[#1F2421] focus:ring-0 focus:outline-none px-3 py-2 text-sm text-[#1F2421] font-mono"
                                    />
                                </div>

                                <div class="border-t border-dashed border-[#8A8478]/30 pt-4 mt-2">
                                    <label class="block text-xs font-mono font-bold text-[#1F2421] uppercase mb-1">
                                        "Authorized Cab Vendor"
                                    </label>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                        <select 
                                            on:change=move |ev| set_vendor_index.set(event_target_value(&ev).parse::<usize>().unwrap_or(0))
                                            class="w-full bg-[#F2EFE6]/30 border border-[#8A8478] focus:border-[#1F2421] focus:ring-0 focus:outline-none px-3 py-2 text-sm text-[#1F2421] font-mono"
                                        >
                                            {vendors.iter().enumerate().map(|(idx, (name, _))| view! {
                                                <option value=idx>{*name}</option>
                                            }).collect::<Vec<_>>()}
                                        </select>
                                        
                                        <div class="bg-[#1F2421]/5 border border-[#8A8478]/40 px-3 py-2 rounded text-[11px] font-mono flex items-center text-[#8A8478] truncate">
                                            {move || vendors[vendor_index.get()].1}
                                        </div>
                                    </div>
                                </div>

                                <button 
                                    type="submit" 
                                    disabled=is_submitting
                                    class="w-full mt-6 bg-[#C8553D] hover:bg-[#C8553D]/95 text-white py-3.5 px-4 font-display font-bold text-lg tracking-wider uppercase border border-[#1F2421] flex items-center justify-center gap-2 cursor-pointer transition-colors"
                                >
                                    "LOG & DISPATCH TO VENDOR"
                                </button>
                            </form>
                        </div>
                    </div>

                    // Vendor Notification Monitor (Mock Email Stream)
                    <div class="bg-[#1F2421] text-[#F2EFE6] border-2 border-[#1F2421] p-6 shadow-md relative">
                        <div class="flex items-center gap-2 border-b border-[#8A8478]/30 pb-3 mb-4">
                            <span class="w-2.5 h-2.5 rounded-full bg-[#5C7A6E]" />
                            <h3 class="font-display text-base font-bold uppercase tracking-wider text-[#F2EFE6]">
                                "VENDOR NOTIFICATION MONITOR"
                            </h3>
                        </div>

                        {move || match selected_email.get() {
                            Some(email) => view! {
                                <div class="font-mono text-xs space-y-3">
                                    <div class="bg-black/30 p-3 rounded border border-white/5 space-y-1">
                                        <div><span class="text-[#8A8478]">"TO:"</span> <span class="text-[#5C7A6E] font-semibold">{email.to}</span></div>
                                        <div><span class="text-[#8A8478]">"SUBJ:"</span> <span class="text-[#F2EFE6]">{email.subject}</span></div>
                                        <div><span class="text-[#8A8478]">"SENT:"</span> <span class="text-[#F2EFE6]">{email.sent_at}</span></div>
                                        <div><span class="text-[#8A8478]">"TICKET:"</span> <span class="text-[#C8553D] font-bold">{email.booking_id}</span></div>
                                    </div>

                                    <div class="bg-black/40 p-4 rounded border border-white/5 whitespace-pre-wrap leading-relaxed text-[#F2EFE6]/80 text-[11px] h-48 overflow-y-auto">
                                        {email.body}
                                    </div>
                                </div>
                            },
                            None => view! {
                                <div class="h-44 flex flex-col items-center justify-center text-center p-4 border border-dashed border-[#8A8478]/30 rounded">
                                    <p class="text-xs text-[#8A8478] font-mono">
                                        "No notifications recorded yet."<br/>"Submit a dispatch ticket to trigger."
                                    </p>
                                </div>
                            }
                        }}

                        // Sent Emails Archive Buttons
                        <div class="mt-4 border-t border-[#8A8478]/20 pt-3">
                            <p class="text-[10px] font-mono text-[#8A8478] mb-2 uppercase">"NOTIFICATION ARCHIVE:"</p>
                            <div class="flex gap-1.5 overflow-x-auto pb-1">
                                {move || sent_emails.get().iter().map(|email| {
                                    let email_clone = email.clone();
                                    let is_selected = selected_email.get().map_or(false, |curr| curr.id == email_clone.id);
                                    view! {
                                        <button
                                            on:click=move |_| set_selected_email.set(Some(email_clone.clone()))
                                            class=format!(
                                                "text-[10px] font-mono px-2 py-1 rounded transition-colors whitespace-nowrap cursor-pointer {}",
                                                if is_selected { "bg-[#C8553D] text-white" } else { "bg-black/20 text-[#8A8478] hover:bg-black/40" }
                                            )
                                        >
                                            {email.booking_id.clone()}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    </div>
                </div>

                // Right Panel: Dispatch Ledger List
                <div class="lg:col-span-7 flex flex-col gap-6">
                    <div class="flex justify-between items-end border-b-2 border-[#1F2421] pb-3">
                        <div>
                            <h2 class="font-display text-2xl font-extrabold tracking-tight uppercase text-[#1F2421]">
                                "DISPATCH LEDGER"
                            </h2>
                            <p class="text-xs text-[#8A8478] font-mono mt-0.5">
                                "ACTIVE TRIP BOOKINGS & DUCKDB TRANSMISSIONS"
                            </p>
                        </div>
                    </div>

                    // Bookings Ledger List
                    <div class="space-y-4">
                        {move || {
                            let list = bookings.get();
                            if list.is_empty() {
                                view! {
                                    <div class="bg-white border border-[#8A8478]/40 border-dashed rounded-lg p-12 text-center text-[#8A8478]">
                                        <p class="font-display text-xl font-bold uppercase text-[#1F2421]">"No Bookings Recorded Yet"</p>
                                    </div>
                                }.into_view()
                            } else {
                                list.into_iter().map(|booking| {
                                    let is_sent = booking.status == "Sent to Vendor";
                                    let id_for_button = booking.id.clone();
                                    let booking_for_slip = booking.clone();
                                    
                                    // Match associated email
                                    let associated_email = sent_emails.get().into_iter().find(|e| e.booking_id == booking_for_slip.id);

                                    view! {
                                        <div class="relative bg-white border border-[#8A8478] pl-7 pr-5 py-4 shadow-sm flex flex-col md:flex-row justify-between items-start md:items-center gap-4 overflow-hidden">
                                            // Perforation tear-off line
                                            <div class="absolute left-0 top-0 bottom-0 w-2.5 border-r border-dashed border-[#8A8478]/40 flex flex-col justify-around py-1 bg-white select-none">
                                                <div class="w-1.5 h-1.5 rounded-full bg-[#F2EFE6] -ml-[4px] border border-[#8A8478]/20" />
                                                <div class="w-1.5 h-1.5 rounded-full bg-[#F2EFE6] -ml-[4px] border border-[#8A8478]/20" />
                                                <div class="w-1.5 h-1.5 rounded-full bg-[#F2EFE6] -ml-[4px] border border-[#8A8478]/20" />
                                            </div>

                                            <div class="flex-1 min-w-0 space-y-1.5">
                                                <div class="flex items-center gap-3">
                                                    <span class="font-mono font-bold text-sm text-[#1F2421] bg-[#1F2421]/5 px-1.5 py-0.5 border border-[#1F2421]/15 rounded">
                                                        {booking.id.clone()}
                                                    </span>
                                                    <span class="font-display font-bold text-lg uppercase tracking-tight text-[#1F2421] truncate">
                                                        {booking.employee_name.clone()}
                                                    </span>
                                                    <span class="font-mono text-[10px] text-[#8A8478] uppercase">
                                                        {booking.department.clone()}
                                                    </span>
                                                </div>

                                                <div class="flex flex-col md:flex-row md:items-center gap-1 md:gap-3 text-xs font-mono text-[#1F2421]/90">
                                                    <div>
                                                        <span class="text-[#8A8478] text-[10px] uppercase">"FROM: "</span>
                                                        <span>{booking.pickup_location.clone()}</span>
                                                    </div>
                                                    <span class="hidden md:inline text-[#C8553D] font-bold">"→"</span>
                                                    <div>
                                                        <span class="text-[#8A8478] text-[10px] uppercase">"TO: "</span>
                                                        <span class="text-[#C8553D] font-semibold">{booking.drop_location.clone()}</span>
                                                    </div>
                                                </div>

                                                <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-[11px] font-mono text-[#8A8478]">
                                                    <span>{booking.date_time.replace("T", " ")}</span>
                                                    <span>"•"</span>
                                                    <span>"VENDOR: " <strong class="text-[#1F2421]">{booking.vendor_name.clone()}</strong></span>
                                                </div>
                                            </div>

                                            // Status and Resend indicator
                                            <div class="flex flex-row md:flex-col items-center md:items-end justify-between md:justify-center w-full md:w-auto shrink-0 border-t md:border-t-0 border-[#8A8478]/20 pt-3 md:pt-0 gap-3">
                                                <div class="flex items-center gap-2">
                                                    <div class="relative flex h-3 w-3">
                                                        <span class=format!("relative inline-flex rounded-full h-3 w-3 {}", if is_sent { "bg-[#5C7A6E]" } else { "bg-[#C8553D]" })></span>
                                                    </div>
                                                    <span class=format!("font-display font-semibold text-xs uppercase tracking-wider {}", if is_sent { "text-[#5C7A6E]" } else { "text-[#C8553D]" })>
                                                        {booking.status.clone()}
                                                    </span>
                                                </div>

                                                <div class="flex gap-2">
                                                    {if let Some(email) = associated_email {
                                                        let email_clone = email.clone();
                                                        view! {
                                                            <button
                                                                on:click=move |_| set_selected_email.set(Some(email_clone.clone()))
                                                                class="text-[10px] font-mono bg-[#1F2421]/10 hover:bg-[#1F2421] hover:text-white text-[#1F2421] px-2.5 py-1 rounded transition-all cursor-pointer uppercase"
                                                            >
                                                                "View Slip"
                                                            </button>
                                                        }.into_view()
                                                    } else {
                                                        view! {}.into_view()
                                                    }}

                                                    {!is_sent}
                                                    {if !is_sent {
                                                        let id_for_click = id_for_button.clone();
                                                        view! {
                                                            <button
                                                                on:click=move |_| mark_as_sent(id_for_click.clone())
                                                                class="text-[10px] font-mono bg-[#C8553D]/10 hover:bg-[#C8553D] hover:text-white text-[#C8553D] px-2.5 py-1 border border-[#C8553D]/30 rounded transition-all cursor-pointer font-bold"
                                                            >
                                                                "Mark Sent"
                                                            </button>
                                                        }.into_view()
                                                    } else {
                                                        view! {}.into_view()
                                                    }}
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>().into_view()
                            }
                        }}
                    </div>
                </div>

            </main>

            <footer class="max-w-7xl mx-auto px-4 mt-16 pt-8 border-t border-[#8A8478]/20 flex flex-col md:flex-row justify-between items-center text-xs font-mono text-[#8A8478] gap-4">
                <div>
                    "CAB LOGGER • RUST WASM CLIENT CONNECTED TO TS + DUCKDB BACKEND"
                </div>
                <div>
                    "PORT: 8090 (FRONTEND) • PORT: 8095 (DB API)"
                </div>
            </footer>
        </div>
    }
}
